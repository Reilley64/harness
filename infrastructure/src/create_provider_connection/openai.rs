use crate::http;
use application::error_context::ResultExt;

use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use application::ports::AuthorizationChallenge;
use application::ports::PortFuture;
use application::ports::PresentAuthorization;
use openidconnect::AccessToken;
use openidconnect::AccessTokenHash;
use openidconnect::ClientId;
use openidconnect::CsrfToken;
use openidconnect::IssuerUrl;
use openidconnect::Nonce;
use openidconnect::PkceCodeChallenge;
use openidconnect::core::CoreClient;
use openidconnect::core::CoreIdToken;
use openidconnect::core::CoreProviderMetadata;
use reqwest::Url;

use super::AuthenticationError;
use super::callback;
use super::settings::OpenaiSettings;
use super::storage;
use super::storage::Registration;
use super::storage::TokenSet;

pub(crate) fn create_provider_connection(
    settings: &OpenaiSettings,
    credentials_path: &Path,
) -> impl Fn(PresentAuthorization) -> PortFuture<Result<domain::ProviderConnection, AuthenticationError>>
+ Send
+ Sync
+ use<> {
    let settings = settings.clone();
    let credentials_path = credentials_path.to_owned();
    move |present_authorization| {
        let settings = settings.clone();
        let credentials_path = credentials_path.clone();
        Box::pin(async move {
            if settings.timeout_seconds == 0 {
                return Err("Authentication timeout must be greater than zero".into());
            }
            tokio::time::timeout(
                Duration::from_secs(settings.timeout_seconds),
                async {
    let issuer = settings.issuer.trim_end_matches('/');
    let issuer_url = Url::parse(issuer)?;
    let loopback = issuer_url
        .host_str()
        .is_some_and(|host| host == "127.0.0.1" || host == "[::1]");
    if !(issuer_url.scheme() == "https" || issuer_url.scheme() == "http" && loopback)
        || !issuer_url.username().is_empty()
        || issuer_url.password().is_some()
        || issuer_url.query().is_some()
        || issuer_url.fragment().is_some()
        || issuer_url.path() != "/"
    {
        return Err(
            "The OAuth issuer must be an HTTPS origin (HTTP is allowed only on loopback)".into(),
        );
    }
    let _lock = storage::lock(&credentials_path).await?;
    let mut credentials = storage::load(&credentials_path).await?;
    let previous = credentials
        .active_client_id
        .as_ref()
        .map(|client_id| {
            credentials
                .registrations
                .get(client_id)
                .map(|registration| (client_id.clone(), registration))
                .ok_or("The active provider registration is missing")
        })
        .transpose()?;
    if previous
        .as_ref()
        .is_some_and(|(_, registration)| registration.issuer != issuer)
    {
        return Err("The stored registration belongs to a different OAuth issuer".into());
    }

    let state = CsrfToken::new_random_len(32);
    let nonce = Nonce::new_random_len(32);
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let mut callback = callback::start(state.secret()).await?;
    let redirect_uri = callback.redirect_uri.clone();
    let mut authorization_url = Url::parse(&format!("{issuer}/api/accounts/authorize"))?;
    authorization_url
        .query_pairs_mut()
        .append_pair(
            "client_id",
            previous
                .as_ref()
                .map(|(id, _)| id.as_str())
                .unwrap_or(&settings.dynamic_client_id),
        )
        .append_pair("ext_agent_host_id", &credentials.host_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", &settings.scopes)
        .append_pair("resource", &settings.resource)
        .append_pair("state", state.secret())
        .append_pair("nonce", nonce.secret())
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", challenge.as_str());
    if previous.is_none() {
        authorization_url
            .query_pairs_mut()
            .append_pair("agent_name_hint", "Harness");
    }
    present_authorization.call(AuthorizationChallenge {
        authorization_url: authorization_url.to_string(),
    })?;
    if settings.open_browser {
        let url = authorization_url.to_string();
        let _ = tokio::task::spawn_blocking(move || webbrowser::open(&url)).await;
    }

    let callback_result = callback
        .receiver
        .recv()
        .await
        .ok_or("The OAuth callback listener stopped")?;
    drop(callback);
    if callback_result.error.is_some() {
        return Err(
            "Authorization was declined or failed in the browser. Start /auth again.".into(),
        );
    }
    let code = callback_result
        .code
        .filter(|code| !code.is_empty())
        .ok_or("The OAuth callback did not include an authorization code")?;
    let client_id = match previous.as_ref() {
        Some((id, _)) => {
            if callback_result
                .client_id
                .as_ref()
                .is_some_and(|returned| returned != id)
            {
                return Err("The OAuth callback changed the selected account's client ID".into());
            }
            id.clone()
        }
        None => callback_result
            .client_id
            .filter(|id| !id.is_empty() && id != &settings.dynamic_client_id)
            .ok_or("Dynamic registration did not return an issued client ID")?,
    };

    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()?;
    let response = http
        .post(format!("{issuer}/api/accounts/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id.as_str()),
            ("code", code.as_str()),
            ("code_verifier", verifier.secret()),
            ("redirect_uri", redirect_uri.as_str()),
            ("resource", settings.resource.as_str()),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!(
            "The provider rejected the code exchange (HTTP {}). Start /auth again.",
            response.status().as_u16()
        )
        .into());
    }
    let tokens: TokenSet = http::json(response, 1_048_576)
        .await
        .context("The provider returned an invalid token response")?;
    if !tokens.token_type.eq_ignore_ascii_case("Bearer")
        || tokens.access_token.is_empty()
        || tokens.refresh_token.is_empty()
        || tokens.expires_in == 0
    {
        return Err("The provider returned an incomplete token response".into());
    }
    for required in ["openid", "resource.invoke", "chatgpt.tokens.use.direct"] {
        if !tokens
            .scope
            .split_whitespace()
            .any(|scope| scope == required)
        {
            return Err(
                "ChatGPT plan usage was not authorized. Start /auth again and grant plan access."
                    .into(),
            );
        }
    }
    let identity_http = http::IdentityClient(http.clone());
    let metadata = CoreProviderMetadata::discover_async(IssuerUrl::new(issuer.to_owned())?, &identity_http)
        .await
        .context("Could not load OpenAI's identity verification keys")?;
    let client =
        CoreClient::from_provider_metadata(metadata, ClientId::new(client_id.clone()), None);
    let id_token: CoreIdToken = tokens
        .id_token
        .parse()
        .context("The provider returned an invalid ID token")?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .context("ID-token signature, issuer, audience, expiry, or nonce validation failed")?;
    if let Some(expected_hash) = claims.access_token_hash() {
        let actual_hash = AccessTokenHash::from_token(
            &AccessToken::new(tokens.access_token.clone()),
            id_token.signing_alg()?,
            id_token.signing_key(&id_token_verifier)?,
        )?;
        if actual_hash != *expected_hash {
            return Err("The access token does not match the verified ID token".into());
        }
    }
    let subject = claims.subject().as_str().to_owned();
    if subject.is_empty() {
        return Err("The ID token does not identify an account".into());
    }
    if previous
        .as_ref()
        .is_some_and(|(_, registration)| registration.subject != subject)
    {
        return Err("The signed-in account differs from the selected registration; existing credentials were not replaced".into());
    }
    let connection_id = previous
        .as_ref()
        .map(|(_, registration)| registration.connection_id.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let saved_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expires_at = saved_at
        .checked_add(tokens.expires_in)
        .ok_or("Invalid access-token expiry")?;
    credentials.registrations.insert(
        client_id.clone(),
        Registration {
            connection_id: connection_id.clone(),
            issuer: issuer.to_owned(),
            subject: subject.clone(),
            tokens,
            saved_at,
            expires_at,
        },
    );
    credentials.active_client_id = Some(client_id);
    storage::save(&credentials_path, &credentials).await?;
    Ok(domain::ProviderConnection {
        id: connection_id,
        provider_id: domain::ProviderId("openai".into()),
    })
                },
            )
            .await
            .context("Authentication timed out. Start /auth again.")?
        })
    }
}
