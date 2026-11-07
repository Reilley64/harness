use std::path::Path;
use std::time::Duration;

use application::error_context::ResultExt;
use application::ports::AuthorizationChallenge;
use application::ports::PortFuture;
use application::ports::PresentAuthorization;
use openidconnect::CsrfToken;
use openidconnect::PkceCodeChallenge;

use crate::cache::encode_bounded;
use crate::create_model_step::checked_url;
use crate::http;
use crate::storage;

use super::AuthenticationError;
use super::callback;
use super::settings::OpenrouterSettings;

#[derive(serde::Serialize, serde::Deserialize)]
struct Record {
    version: u32,
    connection_id: String,
    issuer: String,
    key: String,
}

#[derive(serde::Deserialize)]
struct Response {
    key: String,
}

pub(crate) fn create_provider_connection(
    settings: &OpenrouterSettings,
    path: &Path,
) -> impl Fn(PresentAuthorization) -> PortFuture<Result<domain::ProviderConnection, AuthenticationError>>
+ Send
+ Sync
+ use<> {
    let settings = settings.clone();
    let path = path.to_owned();
    move |present| {
        let settings = settings.clone();
        let path = path.clone();
        Box::pin(async move {
            if settings.timeout_seconds == 0 {
                return Err("Authentication timeout must be greater than zero".into());
            }
            tokio::time::timeout(Duration::from_secs(settings.timeout_seconds), async {
                let issuer = checked_url(settings.issuer.trim_end_matches('/'))?;
                if issuer.path() != "/" {
                    return Err("The OpenRouter issuer must be an origin".into());
                }
                let origin = issuer.origin().ascii_serialization();
                let _lock = storage::lock(&path).await?;
                let previous = match storage::read_private(&path, 1_048_576).await {
                    Ok(bytes) => {
                        let record: Record = tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes)).await??;
                        if record.version != 1 || record.issuer != origin || record.key.is_empty()
                            || uuid::Uuid::parse_str(&record.connection_id).is_err() {
                            return Err("Invalid or incompatible stored OpenRouter credentials".into());
                        }
                        Some(record)
                    }
                    Err(error) if error.downcast_ref::<std::io::Error>().is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) => None,
                    Err(error) => return Err(error),
                };
                let state = CsrfToken::new_random_len(32);
                let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
                let mut callback = callback::start(state.secret()).await?;
                let mut url = issuer.join("auth")?;
                url.query_pairs_mut()
                    .append_pair("callback_url", &callback.redirect_uri)
                    .append_pair("state", state.secret())
                    .append_pair("code_challenge", challenge.as_str())
                    .append_pair("code_challenge_method", "S256");
                present.call(AuthorizationChallenge { authorization_url: url.to_string() })?;
                if settings.open_browser {
                    let url = url.to_string();
                    let _ = tokio::task::spawn_blocking(move || webbrowser::open(&url)).await;
                }
                let result = callback.receiver.recv().await.ok_or("The authorization callback listener stopped")?;
                drop(callback);
                if result.error.is_some() {
                    return Err("OpenRouter authorization was declined; start /auth again".into());
                }
                let code = result.code.filter(|code| !code.is_empty() && code.len() <= 8192)
                    .ok_or("The authorization callback has no valid code")?;
                let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
                    .timeout(Duration::from_secs(15)).build()?;
                let response = client.post(issuer.join("api/v1/auth/keys")?)
                    .json(&serde_json::json!({"code":code, "code_verifier":verifier.secret(), "code_challenge_method":"S256"}))
                    .send().await.context("Could not contact OpenRouter for authorization")?;
                if !response.status().is_success() {
                    return Err("OpenRouter rejected the authorization code exchange; start /auth again".into());
                }
                let response: Response = http::json(response, 1_048_576).await
                    .context("OpenRouter returned an invalid credential response")?;
                if response.key.is_empty() || response.key.len() > 16384 || response.key.chars().any(char::is_control) {
                    return Err("OpenRouter returned an invalid API key".into());
                }
                let id = previous.map(|record| record.connection_id).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let record = Record { version: 1, connection_id: id.clone(), issuer: origin, key: response.key };
                let bytes = tokio::task::spawn_blocking(move || encode_bounded(&record, 1_048_576)).await??;
                storage::replace(&path, &bytes).await?;
                Ok(domain::ProviderConnection { id, provider_id: domain::ProviderId("openrouter".into()) })
            }).await.context("OpenRouter authorization timed out; start /auth again")?
        })
    }
}
