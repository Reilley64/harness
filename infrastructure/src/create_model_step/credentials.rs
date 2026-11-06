use application::error_context::ResultExt;

use std::path::Path;
use std::time::Duration;

use openidconnect::AccessToken;
use openidconnect::AccessTokenHash;
use openidconnect::ClientId;
use openidconnect::IssuerUrl;
use openidconnect::Nonce;
use openidconnect::core::CoreClient;
use openidconnect::core::CoreIdToken;
use openidconnect::core::CoreProviderMetadata;

use super::record as storage;
use super::record::TokenSet;
use crate::conversations::now;
use crate::create_provider_connection::settings::OpenaiSettings;
use crate::http;

use super::ModelStepError;

#[derive(serde::Deserialize)]
struct RefreshedTokens {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
    id_token: Option<String>,
    earliest_refresh_at: Option<serde_json::Value>,
}

pub(super) async fn access_token(
    path: &Path,
    connection_id: &str,
    settings: &OpenaiSettings,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<String, ModelStepError> {
    if !tokio::fs::try_exists(path).await? {
        return Err("The saved provider connection is unavailable; sign in with /auth".into());
    }
    let _guard = loop {
        if cancellation.is_cancelled() {
            return Err("Reply was cancelled".into());
        }
        match storage::lock(path).await {
            Ok(guard) => break guard,
            Err(error)
                if error
                    .source()
                    .and_then(|source| source.downcast_ref::<std::fs::TryLockError>())
                    .is_some_and(|error| matches!(error, std::fs::TryLockError::WouldBlock)) =>
            {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => return Err(error),
        }
    };
    let mut credentials = storage::load(path).await?;
    let client_id = credentials
        .active_client_id
        .clone()
        .ok_or("Sign in with /auth before sending messages")?;
    let registration = credentials
        .registrations
        .get_mut(&client_id)
        .ok_or("The provider registration is missing")?;
    if registration.connection_id != connection_id
        || registration.issuer != settings.issuer.trim_end_matches('/')
    {
        return Err(
            "This conversation belongs to a different provider connection or issuer".into(),
        );
    }
    let current = now()?;
    if registration.expires_at > current.saturating_add(60) {
        return Ok(registration.tokens.access_token.clone());
    }
    if let Some(earliest) = &registration.tokens.earliest_refresh_at {
        let earliest = match earliest {
            serde_json::Value::Number(number) => {
                number.as_u64().ok_or("Invalid token refresh schedule")?
            }
            serde_json::Value::String(value) => u64::try_from(
                chrono::DateTime::parse_from_rfc3339(value)
                    .context("Invalid token refresh schedule")?
                    .timestamp(),
            )
            .context("Invalid token refresh schedule")?,
            _ => return Err("Invalid token refresh schedule".into()),
        };
        if earliest > current {
            if registration.expires_at > current {
                return Ok(registration.tokens.access_token.clone());
            }
            return Err("The provider has not permitted refresh yet; try again later".into());
        }
    }
    if cancellation.is_cancelled() {
        return Err("Reply was cancelled".into());
    }
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()?;
    let issuer = settings.issuer.trim_end_matches('/');
    let token_url = super::checked_url(&format!("{issuer}/api/accounts/oauth/token"))?;
    let response = http
        .post(token_url)
        .timeout(Duration::from_secs(15))
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id.as_str()),
            ("refresh_token", registration.tokens.refresh_token.as_str()),
            ("resource", settings.resource.as_str()),
        ])
        .send()
        .await
        .context("Could not contact the provider to refresh credentials")?;
    if !response.status().is_success() {
        return Err(format!(
            "Credential refresh failed (HTTP {}); sign in again with /auth",
            response.status().as_u16()
        )
        .into());
    }
    let refreshed: RefreshedTokens = http::json(response, 1_048_576)
        .await
        .context("The provider returned an invalid refresh response")?;
    if !refreshed.token_type.eq_ignore_ascii_case("Bearer")
        || refreshed.access_token.is_empty()
        || refreshed.refresh_token.is_empty()
        || refreshed.expires_in == 0
    {
        return Err("The provider returned incomplete refreshed credentials".into());
    }
    let scope = refreshed
        .scope
        .unwrap_or_else(|| registration.tokens.scope.clone());
    if !["openid", "resource.invoke", "chatgpt.tokens.use.direct"]
        .iter()
        .all(|required| scope.split_whitespace().any(|value| value == *required))
    {
        return Err("Refreshed credentials do not authorize ChatGPT plan usage".into());
    }
    if let Some(raw) = &refreshed.id_token {
        let identity_http = http::IdentityClient(http.clone());
        let metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(issuer.to_owned())?,
            &identity_http,
        )
        .await
        .context("Could not verify refreshed identity")?;
        let client =
            CoreClient::from_provider_metadata(metadata, ClientId::new(client_id.clone()), None);
        let token: CoreIdToken = raw.parse().context("Invalid refreshed identity token")?;
        let verifier = client.id_token_verifier();
        let claims = token
            .claims(&verifier, |_: Option<&Nonce>| Ok(()))
            .context("Refreshed identity verification failed")?;
        if claims.subject().as_str() != registration.subject {
            return Err("Refresh changed the connected account".into());
        }
        if let Some(expected) = claims.access_token_hash() {
            let actual = AccessTokenHash::from_token(
                &AccessToken::new(refreshed.access_token.clone()),
                token.signing_alg()?,
                token.signing_key(&verifier)?,
            )?;
            if expected != &actual {
                return Err("Refreshed access token does not match its identity token".into());
            }
        }
    }
    let saved_at = now()?;
    registration.expires_at = saved_at
        .checked_add(refreshed.expires_in)
        .ok_or("Invalid refreshed expiry")?;
    registration.saved_at = saved_at;
    registration.tokens = TokenSet {
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token,
        token_type: refreshed.token_type,
        expires_in: refreshed.expires_in,
        scope,
        id_token: refreshed
            .id_token
            .unwrap_or_else(|| registration.tokens.id_token.clone()),
        earliest_refresh_at: refreshed.earliest_refresh_at,
    };
    let access = registration.tokens.access_token.clone();
    storage::save(path, &credentials).await?;
    Ok(access)
}
