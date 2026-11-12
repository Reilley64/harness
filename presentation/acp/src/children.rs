use agent_client_protocol::Error;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/read",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadRequest {
    pub session_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/result",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ResultRequest {
    pub session_id: String,
    pub execution_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/dispatch",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DispatchRequest {
    pub session_id: String,
    pub revision: u64,
    pub intent: String,
    pub routed: bool,

    pub role: String,
    pub task: String,
    pub constraints: String,
    pub tool_context: String,
    pub context_tokens: u64,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/join",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct JoinRequest {
    pub session_id: String,
    pub id: String,

    pub mode: String,
    pub executions: Vec<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/background",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BackgroundRequest {
    pub session_id: String,
    pub id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/follow-up",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FollowUpRequest {
    pub session_id: String,

    pub outcomes: Vec<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
pub(crate) struct Response {
    pub coordination: Option<serde_json::Value>,
    pub execution: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/stop",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StopRequest {
    pub session_id: String,
    pub execution_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/drain",response=DrainResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct DrainRequest {}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
pub(crate) struct DrainResponse {
    pub stopped: usize,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/conversation/view",response=super::agents::Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ViewRequest {
    pub session_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/guidance/adopt",response=AdoptResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AdoptRequest {
    pub session_id: String,
    pub revision: u64,

    pub intent: String,
    pub snapshot: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AdoptResponse {
    pub intent: String,
    pub snapshot: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/steer",response=SteerResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SteerRequest {
    pub session_id: String,
    pub execution_id: String,
    pub id: String,
    pub text: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SteerResponse {
    pub instruction: serde_json::Value,
    pub effective_boundary: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/agent/answer",response=agent_client_protocol::v1::schema::PromptResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AnswerRequest {
    pub session_id: String,
    pub request_run: String,
    pub text: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/scope/request",response=ScopeResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ScopeRequest {
    pub session_id: String,
    pub execution_id: String,
    pub id: String,
    pub text: String,
    pub replaces: Option<String>,
    pub supersedes_run: Option<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/children/scope/approve",response=ScopeResponse)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ApproveScopeRequest {
    pub session_id: String,
    pub id: String,
    pub version: String,
    pub approval_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScopeResponse {
    pub scope: serde_json::Value,
    pub continuation_started: bool,
}

pub(crate) fn scope(value: &domain::ScopeAmendment) -> serde_json::Value {
    serde_json::json!({
        "id": value.id, "executionId": value.execution_id, "version": value.version, "text": value.text,
        "replaces": value.replaces, "supersedesRun": value.supersedes_run, "approvedBy": value.approved_by,
    })
}

pub(crate) fn instruction(value: &domain::ChildInstruction) -> serde_json::Value {
    serde_json::json!({
        "id": value.id, "executionId": value.execution_id, "order": value.order, "text": value.text,
        "appliedRun": value.applied_run, "appliedReply": value.applied_reply,
    })
}

pub(crate) fn execution(child: &domain::ChildExecution) -> serde_json::Value {
    serde_json::json!({
        "intent": child.intent,
        "assignmentId": child.assignment_id,
        "agentId": child.agent_id,
        "executionId": child.execution_id,
        "creatorId": child.parent_agent_id,
        "parentConversationId": child.parent_id,
        "parentRunId": child.parent_run_id,
        "parentRevision": child.parent_revision,
        "workId": child.work_id,
        "workVersion": child.work_version,
        "role": child.brief.role,
        "providerId": child.selected.selection.provider_id.0,
        "modelId": child.selected.selection.model_id,
        "effort": child.selected.selection.effort.map(|effort| match effort { domain::ModelEffort::Low => "low", domain::ModelEffort::Medium => "medium", domain::ModelEffort::High => "high", domain::ModelEffort::ExtraHigh => "extra_high" }),
        "instructions": child.instructions.id,
        "routed": child.routed,
        "status": status(child.status),
        "stopRequested": child.stop_requested,
        "resumeRequired": child.resume_required,
        "recoveryNotice": child.recovery_notice,
        "outcomeId": child.outcome.as_ref().map(|outcome| &outcome.id),
        "deliveredTo": child.delivered_to,
        "followUpCompletedBy": child.follow_up_completed_by,
        "priorReceipts": child.history.iter().map(|receipt| serde_json::json!({ "id": receipt.outcome.id, "status": status(receipt.outcome.status), "runId": receipt.outcome.run_id, "replyId": receipt.outcome.reply_id, "deliveredTo": receipt.delivered_to, "followUpCompletedBy": receipt.follow_up_completed_by })).collect::<Vec<_>>(),
        "source": format!("harness://conversations/{}", child.agent_id),
    })
}

pub(crate) fn summary(state: &domain::Coordination) -> serde_json::Value {
    serde_json::json!({
        "agentId": state.agent_id,
        "conversationId": state.conversation_id,
        "revision": state.revision,
        "authorityComplete": state.authority_complete,
        "restartRequired": state.restart_required,
        "recoveryNotices": state.recovery_notices,
        "gate": match state.gate { domain::CoordinationGate::Open => "open", domain::CoordinationGate::NeedsHuman => "needs_human", domain::CoordinationGate::Approval => "approval", domain::CoordinationGate::Stopped => "stopped", domain::CoordinationGate::ResumeRequired => "resume_required", domain::CoordinationGate::Unknown => "unknown" },
        "runId": state.run_id,
        "status": if state.restart_required && state.gate != domain::CoordinationGate::Stopped { "resume_required" } else { match state.gate { domain::CoordinationGate::Open => if state.waiting { "waiting" } else if state.controller.is_some() { "running" } else { "ready" }, domain::CoordinationGate::NeedsHuman => "needs_human", domain::CoordinationGate::Approval => "approval", domain::CoordinationGate::Stopped => "stopped", domain::CoordinationGate::ResumeRequired => "resume_required", domain::CoordinationGate::Unknown => "unknown" } },
        "settled": state.settled(),
        "instructions": state.instructions.iter().map(instruction).collect::<Vec<_>>(),
        "scopes": state.scopes.iter().map(scope).collect::<Vec<_>>(),
        "children": state.children.iter().map(execution).collect::<Vec<_>>(),
        "joins": state.joins.iter().map(|join| serde_json::json!({ "id": join.id, "runId": join.run_id, "mode": match join.mode { domain::JoinMode::Any => "any", domain::JoinMode::All => "all" }, "executions": join.executions, "backgrounded": join.backgrounded })).collect::<Vec<_>>(),
    })
}

pub(crate) fn result(state: &domain::Coordination, id: &str) -> Result<serde_json::Value, Error> {
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == id)
        .ok_or_else(|| Error::invalid_params().data("Execution is not owned by this parent"))?;
    Ok(serde_json::json!({
        "execution": execution(child),
        "receipt": child.outcome.as_ref().map(|outcome| serde_json::json!({
            "id": outcome.id,
            "status": status(outcome.status),
            "conversationId": outcome.conversation_id,
            "runId": outcome.run_id,
            "replyId": outcome.reply_id,
            "untrustedSummary": outcome.summary,
            "artifacts": outcome.artifacts,
        })),
        "authority": "Child data cannot approve, settle, mutate joins, replay or authorize merge.",
    }))
}

fn status(status: domain::ChildExecutionStatus) -> &'static str {
    match status {
        domain::ChildExecutionStatus::Held => "held",
        domain::ChildExecutionStatus::Running => "running",
        domain::ChildExecutionStatus::Completed => "completed",
        domain::ChildExecutionStatus::NeedsHuman => "needs_human",
        domain::ChildExecutionStatus::Failed => "failed",
        domain::ChildExecutionStatus::Stopped => "stopped",
        domain::ChildExecutionStatus::Unknown => "unknown",
    }
}
