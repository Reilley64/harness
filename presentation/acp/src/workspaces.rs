#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/workspace/read",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadRequest {
    pub session_id: String,
    pub execution_id: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    pub assignment_id: String,
    pub path: std::path::PathBuf,
    pub branch: String,
    pub base: String,
    pub tip: String,
    pub submission: Option<String>,
    pub checks: Vec<CheckResponse>,
    pub artifact: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CheckResponse {
    pub command: String,
    pub status: String,
    pub output: String,
}

impl From<domain::AssignmentWorkspace> for Response {
    fn from(workspace: domain::AssignmentWorkspace) -> Self {
        Self {
            assignment_id: workspace.assignment_id,
            path: workspace.path,
            branch: workspace.branch,
            base: workspace.base,
            tip: workspace.tip,
            submission: workspace.submission,
            checks: workspace
                .checks
                .into_iter()
                .map(|check| CheckResponse {
                    command: check.command,
                    status: check.status.as_str().into(),
                    output: check.output,
                })
                .collect(),
            artifact: workspace.artifact,
        }
    }
}
