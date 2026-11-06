#[derive(
    Debug,
    Clone,
    Default,
    agent_client_protocol::JsonRpcRequest,
    serde::Serialize,
    serde::Deserialize,
)]
#[request(method = "_harness/providers", response = ProvidersResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProvidersRequest {}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcResponse, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct ProvidersResponse {
    pub providers: Vec<ProviderSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProviderSummary {
    pub id: String,
    pub name: String,
}
