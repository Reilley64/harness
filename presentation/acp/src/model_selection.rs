use agent_client_protocol::Error;

use crate::models::ModelEffort;
use crate::models::ModelSummary;

#[derive(
    Debug,
    Clone,
    Default,
    agent_client_protocol::JsonRpcRequest,
    serde::Serialize,
    serde::Deserialize,
)]
#[request(method = "_harness/model-selection/read", response = ModelSelectionResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReadModelSelectionRequest {}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcRequest, serde::Serialize, serde::Deserialize,
)]
#[request(method = "_harness/model-selection/update", response = ModelSelectionResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateModelSelectionRequest {
    provider_id: String,
    model_id: String,
    effort: Option<ModelEffort>,
}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcResponse, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct ModelSelectionResponse {
    pub selection: Option<SelectedModelSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SelectedModelSummary {
    model: ModelSummary,
    effort: Option<ModelEffort>,
}

impl From<domain::SelectedModel> for SelectedModelSummary {
    fn from(selected: domain::SelectedModel) -> Self {
        Self {
            model: selected.model.into(),
            effort: selected.effort.map(Into::into),
        }
    }
}

impl From<UpdateModelSelectionRequest> for domain::ModelSelection {
    fn from(request: UpdateModelSelectionRequest) -> Self {
        Self {
            provider_id: request.provider_id.into(),
            model_id: request.model_id,
            effort: request.effort.map(Into::into),
        }
    }
}

pub(crate) fn error(error: impl std::error::Error + 'static) -> Error {
    Error::new(-32005, "Could not save or restore the model selection")
        .data(application::error_context::diagnostic(&error))
}
