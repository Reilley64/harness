use crate::tools;

pub(crate) fn role(role: Option<String>) -> Result<Option<domain::AgentRole>, &'static str> {
    match role.as_deref() {
        None => Ok(None),
        Some("setup") => Ok(Some(domain::AgentRole::Setup)),
        Some("griller") => Ok(Some(domain::AgentRole::Griller)),
        _ => Err("Only Setup and Griller have execution paths"),
    }
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/setup/read",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadRequest {
    pub cwd: std::path::PathBuf,
    pub session_id: Option<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    workspace: std::path::PathBuf,
    roots: Vec<String>,
    missing: Vec<String>,
    triage: bool,
    revision: Option<u64>,
    proposals: Vec<serde_json::Value>,
}

impl From<application::use_cases::read_setup::Output> for Response {
    fn from(output: application::use_cases::read_setup::Output) -> Self {
        Self {
            workspace: output.status.workspace,
            roots: output.status.roots,
            missing: output.status.missing,
            triage: output.status.triage,
            revision: output.conversation.as_ref().map(|c| c.revision),
            proposals: output
                .conversation
                .into_iter()
                .flat_map(|c| c.messages)
                .flat_map(|m| {
                    let id = m.id;
                    m.steps
                        .into_iter()
                        .flat_map(|s| s.calls)
                        .filter(|c| c.draft.is_some())
                        .map(move |c| tools::summary(&id, &c))
                })
                .collect(),
        }
    }
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/setup/update",response=super::agents::Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateRequest {
    pub session_id: String,
    pub revision: u64,
    pub proposal: String,
    pub confirm: bool,
}
