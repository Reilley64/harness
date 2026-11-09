use crate::models::ModelEffort;
use serde_json::json;
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/work/read",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadRequest {
    pub id: String,
}
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/work/approve",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ApproveRequest {
    pub id: String,
    pub revision: u64,
    pub version: String,
    pub intent: String,
    pub session_revision: u64,
    pub provider_id: String,
    pub model_id: String,
    pub effort: Option<ModelEffort>,
}
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    pub work: Option<serde_json::Value>,
    pub conversation_id: Option<String>,
}
pub(crate) fn summary(w: &domain::ApprovedWork) -> serde_json::Value {
    json!({"id":w.id,"creatorId":w.creator_id,"sessionId":w.conversation_id,"workspace":w.workspace,"revision":w.revision,
 "publications":w.publications.iter().map(|p|json!({"intent":p.intent,"version":p.proposal.version,"document":p.proposal.document})).collect::<Vec<_>>(),
 "approvals":w.approvals.iter().map(|a|json!({"intent":a.intent,"version":a.version,"revision":a.revision,"providerId":a.selection.provider_id.0,"modelId":a.selection.model_id,"effort":a.selection.effort.map(|e|match e {domain::ModelEffort::Low=>"low",domain::ModelEffort::Medium=>"medium",domain::ModelEffort::High=>"high",domain::ModelEffort::ExtraHigh=>"extra_high"}),"conversationId":a.coordinator_id,"approvedAt":a.approved_at,"delivered":a.delivered,"current":w.current().is_ok_and(|p|p.version==a.version)})).collect::<Vec<_>>(),
 "unavailable":["implementer_dispatch","managed_workspaces","review","integration","target_merge"],
 "complete":false})
}
