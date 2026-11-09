use std::path::Path;

use application::error_context::ResultExt;
use application::ports::PresentTurnEvent;
use tokio_util::sync::CancellationToken;

use crate::create_provider_connection::settings::OpenaiSettings as AuthenticationSettings;

use super::ModelStepFuture;
use super::OpenaiSettings;
use super::context;
use super::credentials;
use super::responses;

pub(crate) fn create_model_step(
    settings: &OpenaiSettings,
    authentication: &AuthenticationSettings,
    credentials_path: &Path,
    read_model_allowlist: application::ports::ReadModelAllowlist,
    read_models: application::ports::ReadModels,
    read_work: application::ports::ReadWork,
    read_coordination: application::ports::ReadCoordination,
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
    let authentication = authentication.clone();
    let path = credentials_path.to_owned();
    responses::create_model_step(
        &settings.base_url,
        settings.timeout_seconds,
        move |conversation, tools, finalizing, cancellation| {
            let authentication = authentication.clone();
            let path = path.clone();
            Box::pin(async move {
                let token = credentials::access_token(&path, &conversation.provider_connection_id, &authentication, &cancellation).await
                .context("Authentication blocked: the saved connection could not authorize inference; sign in with /auth")?;
                let body = context::request(conversation, tools, finalizing).await?;
                Ok((token, body))
            })
        },
        read_model_allowlist,
        read_models,
        read_work,
        read_coordination,
    )
}
