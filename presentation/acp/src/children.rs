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
        "role": "research",
        "providerId": child.selected.selection.provider_id.0,
        "modelId": child.selected.selection.model_id,
        "effort": child.selected.selection.effort.map(|effort| match effort { domain::ModelEffort::Low => "low", domain::ModelEffort::Medium => "medium", domain::ModelEffort::High => "high", domain::ModelEffort::ExtraHigh => "extra_high" }),
        "instructions": child.instructions.id,
        "routed": child.routed,
        "status": status(child.status),
        "outcomeId": child.outcome.as_ref().map(|outcome| &outcome.id),
        "deliveredTo": child.delivered_to,
        "followUpCompletedBy": child.follow_up_completed_by,
        "source": format!("harness://conversations/{}", child.agent_id),
    })
}

pub(crate) fn summary(state: &domain::Coordination) -> serde_json::Value {
    serde_json::json!({
        "agentId": state.agent_id,
        "conversationId": state.conversation_id,
        "revision": state.revision,
        "gate": match state.gate { domain::CoordinationGate::Open => "open", domain::CoordinationGate::NeedsHuman => "needs_human", domain::CoordinationGate::Approval => "approval", domain::CoordinationGate::Stopped => "stopped", domain::CoordinationGate::ResumeRequired => "resume_required", domain::CoordinationGate::Unknown => "unknown" },
        "runId": state.run_id,
        "status": match state.gate { domain::CoordinationGate::Open => if state.waiting { "waiting" } else if state.controller.is_some() { "running" } else { "ready" }, domain::CoordinationGate::NeedsHuman => "needs_human", domain::CoordinationGate::Approval => "approval", domain::CoordinationGate::Stopped => "stopped", domain::CoordinationGate::ResumeRequired => "resume_required", domain::CoordinationGate::Unknown => "unknown" },
        "settled": state.settled(),
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
