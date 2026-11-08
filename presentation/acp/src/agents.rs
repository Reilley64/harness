use crate::conversations;
use agent_client_protocol::v1::schema::Meta;
use agent_client_protocol::v1::schema::PromptResponse;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/agent/read",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadRequest {
    #[serde(default)]
    pub role: Option<String>,
    pub cwd: std::path::PathBuf,
}
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/agent/start",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StartRequest {
    #[serde(default)]
    pub role: Option<String>,
    pub cwd: std::path::PathBuf,
    pub selection: Meta,
}
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
pub(crate) struct Response {
    pub conversation: Option<serde_json::Value>,
}
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/agent/resume",response=PromptResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ResumeRequest {
    pub session_id: String,
}
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/agent/acknowledge",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AcknowledgeRequest {
    pub session_id: String,
}
pub(crate) fn response(conversation: &domain::Conversation) -> Response {
    Response {
        conversation: conversations::metadata(conversation).remove("harness/conversation"),
    }
}
