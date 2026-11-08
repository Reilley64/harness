use crate::models::ModelSummary;
use serde_json::json;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/decision/read", response = Response)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReadRequest {
    pub id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/decision/select", response = Response)]
#[serde(deny_unknown_fields)]
pub(crate) struct SelectRequest {
    pub id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/decision/stop", response = Response)]
#[serde(deny_unknown_fields)]
pub(crate) struct StopRequest {
    pub id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
pub(crate) struct Response {
    pub decision: Option<serde_json::Value>,
    pub blocked: Option<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/deciders", response = DecidersResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecidersRequest {}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/decider/update", response = DecidersResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateRequest {
    pub model_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
pub(crate) struct DecidersResponse {
    pub models: Vec<ModelSummary>,
    pub selected: Option<String>,
}

fn pair(p: &domain::ModelSelection) -> serde_json::Value {
    json!({"providerId":p.provider_id.0,"modelId":p.model_id,"effort":p.effort.map(|e| match e { domain::ModelEffort::Low => "low", domain::ModelEffort::Medium => "medium", domain::ModelEffort::High => "high", domain::ModelEffort::ExtraHigh => "extra_high" })})
}

fn candidate(c: &domain::DecisionCandidate) -> serde_json::Value {
    let mut p = pair(&c.selection);
    p["key"] = json!(c.key);
    p["connectionId"] = json!(c.connection_id);
    p["canonicalId"] = json!(c.model.canonical_id);
    p["contextWindow"] = json!(c.model.context_window);
    p["pricing"] = c
        .model
        .pricing
        .as_ref()
        .map(|p| json!({"input":p.input,"cachedInput":p.cached_input,"output":p.output}))
        .unwrap_or(serde_json::Value::Null);
    p
}

pub(crate) fn summary(d: &domain::AssignmentDecision) -> serde_json::Value {
    json!({
        "id":d.id,"revision":d.revision,"finishedAt":d.finished_at,"parentId":d.parent_id,"parentAgentId":d.parent_agent_id,"parentRevision":d.parent_revision,
        "parent":pair(&d.parent),"parentConnectionId":d.parent_connection_id,"workId":d.work_id,"workIntent":d.work_intent,"workVersion":d.work_version,
        "brief":{"role":d.brief.role,"task":d.brief.task,"constraints":d.brief.constraints,"toolContext":d.brief.tool_context,"contextTokens":d.brief.context_tokens},
        "phase":match d.phase { domain::DecisionPhase::Prepared => "prepared", domain::DecisionPhase::Pending => "pending", domain::DecisionPhase::Selected => "selected", domain::DecisionPhase::Fallback => "fallback", domain::DecisionPhase::Configuration => "configuration", domain::DecisionPhase::Unsupported => "unsupported", domain::DecisionPhase::Stopped => "stopped" },
        "request":d.request.as_ref().map(|r| json!({"modelId":r.model.id,"canonicalId":r.model.canonical_id,"movingAlias":r.model.id.starts_with('~'),"connectionId":r.connection_id,"policy":r.policy,"capturedAt":r.captured_at,"candidates":r.candidates.iter().map(candidate).collect::<Vec<_>>()})),
        "response":d.response.as_ref().map(|r| json!({"choice":r.choice,"confidence":r.confidence,"probabilities":r.probabilities,"resolvedModel":r.resolved_model,"servingProvider":r.serving_provider,"inputTokens":r.input_tokens,"outputTokens":r.output_tokens,"cost":r.cost})),
        "selected":d.selected.as_ref().map(candidate),"reason":d.reason,"workerLaunched":false
    })
}
