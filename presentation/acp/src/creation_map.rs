use crate::children;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/agents/map",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request {
    pub cwd: std::path::PathBuf,
    pub after: Option<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
pub(crate) struct Response {
    pub agents: Vec<serde_json::Value>,
    pub next: Option<String>,
}

pub(crate) fn response(output: application::use_cases::read_creation_map::Output) -> Response {
    let agents = output.agents.iter().filter_map(|(conversation, coordination, execution)| {
        let agent = conversation.agent.as_ref()?;
        let state = coordination.as_ref().map(children::summary);
        let mut pending_results = 0usize;
        let mut required_follow_up = 0usize;
        if let Some(coordination) = coordination {
            for child in &coordination.children {
                for (_, delivered, completed) in child.receipts() {
                    pending_results += usize::from(delivered.is_none());
                    required_follow_up += usize::from(completed.is_none());
                }
            }
        }
        let saved_status = conversation.runs.last().map(|run| match run.status {
            domain::RunStatus::Running => "running",
            domain::RunStatus::NeedsHuman => "needs_human",
            domain::RunStatus::Stopped => "stopped",
            domain::RunStatus::Failed => "failed",
            domain::RunStatus::Interrupted => "interrupted",
        });
        Some(serde_json::json!({
            "id": agent.id.0,
            "conversationId": conversation.id,
            "creatorId": conversation.research.as_ref().map(|research| &research.parent_agent_id).or_else(|| conversation.assignment.as_ref().map(|assignment| &assignment.creator_id)),
            "role": match agent.role { domain::AgentRole::Setup => "setup", domain::AgentRole::Griller => "griller", domain::AgentRole::Research => "research", domain::AgentRole::ChiefOfStaff => "chief_of_staff", domain::AgentRole::Decider => "decider", domain::AgentRole::Implementer => "implementer", domain::AgentRole::Reviewer => "reviewer", domain::AgentRole::Retro => "retro" },
            "revision": conversation.revision,
            "createdAt": conversation.created_at,
            "savedStatus": saved_status,
            "runId": conversation.runs.last().map(|run| &run.id),
            "resumeRequired": conversation.resume_required,
            "coordination": state.as_ref().map(|state| serde_json::json!({"status":state["status"],"gate":state["gate"],"authorityComplete":state["authorityComplete"],"restartRequired":state["restartRequired"],"revision":state["revision"]})),
            "execution": execution.as_ref().map(children::execution),
            "pendingResults": pending_results,
            "requiredFollowUp": required_follow_up,
        }))
    }).collect();
    Response {
        agents,
        next: output.next,
    }
}
