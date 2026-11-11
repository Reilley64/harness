#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/execution/stop",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request {
    pub session_id: String,
    pub execution_id: Option<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    pub coordination: serde_json::Value,
    pub executions: Vec<String>,
}
