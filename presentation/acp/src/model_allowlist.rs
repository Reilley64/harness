#[derive(
    Debug,
    Clone,
    Default,
    agent_client_protocol::JsonRpcRequest,
    serde::Serialize,
    serde::Deserialize,
)]
#[request(method = "_harness/model-allowlist/read", response = ModelAllowlistResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReadModelAllowlistRequest {}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcRequest, serde::Serialize, serde::Deserialize,
)]
#[request(method = "_harness/model-allowlist/update", response = ModelAllowlistResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateModelAllowlistRequest {
    pub patterns: Vec<String>,
}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcResponse, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct ModelAllowlistResponse {
    pub patterns: Vec<String>,
}

impl From<domain::ModelAllowlist> for ModelAllowlistResponse {
    fn from(policy: domain::ModelAllowlist) -> Self {
        Self {
            patterns: policy
                .patterns()
                .iter()
                .map(|pattern| pattern.as_str().to_owned())
                .collect(),
        }
    }
}

pub(crate) fn error(error: impl std::error::Error + 'static) -> agent_client_protocol::Error {
    agent_client_protocol::Error::new(-32006, "Model allowlist configuration failed")
        .data(application::error_context::diagnostic(&error))
}
