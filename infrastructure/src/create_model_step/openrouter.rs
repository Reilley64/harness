use std::path::Path;

use application::error_context::ResultExt;
use application::ports::PresentTurnEvent;
use tokio_util::sync::CancellationToken;

use crate::storage;

use super::ModelStepFuture;
use super::OpenrouterSettings;
use super::checked_url;
use super::context;
use super::responses;

#[derive(serde::Deserialize)]
struct Record {
    version: u32,
    connection_id: String,
    issuer: String,
    key: String,
}

pub(crate) fn create_model_step(
    settings: &OpenrouterSettings,
    issuer: &str,
    credentials_path: &Path,
    read_model_allowlist: application::ports::ReadModelAllowlist,
    read_models: application::ports::ReadModels,
) -> impl Fn(
    domain::Conversation,
    Vec<domain::Tool>,
    bool,
    PresentTurnEvent,
    CancellationToken,
) -> ModelStepFuture
+ Send
+ Sync
+ use<> {
    let path = credentials_path.to_owned();
    let base_url = settings.base_url.clone();
    let issuer = issuer.to_owned();
    responses::create_model_step(
        &settings.base_url,
        settings.timeout_seconds,
        move |conversation, tools, finalizing, cancellation| {
            let path = path.clone();
            let base_url = base_url.clone();
            let issuer = issuer.clone();
            Box::pin(async move {
                let bytes = tokio::select! {
                    result = storage::read_private(&path, 1_048_576) => result.context("Authentication blocked: sign in with /auth for OpenRouter")?,
                    _ = cancellation.cancelled() => return Err("Model generation was cancelled".into()),
                };
                let record: Record =
                    tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes)).await??;
                let destination = checked_url(&base_url)?;
                let authority = checked_url(&issuer)?;
                if authority.path() != "/"
                    || record.version != 1
                    || record.connection_id != conversation.provider_connection_id
                    || record.key.is_empty()
                    || record.key.len() > 16384
                    || record.key.chars().any(char::is_control)
                    || record.issuer != authority.origin().ascii_serialization()
                    || destination.origin() != authority.origin()
                {
                    return Err("OpenRouter credentials cannot authorize this connection or origin").context("Authentication blocked: the saved OpenRouter connection does not match; check configuration and /auth").map_err(Into::into);
                }
                let mut body = context::request(conversation, tools, finalizing).await?;
                body["provider"] =
                    serde_json::json!({"allow_fallbacks":false,"require_parameters":true});
                body["plugins"] = serde_json::json!([]);
                Ok((record.key, body))
            })
        },
        read_model_allowlist,
        read_models,
    )
}
