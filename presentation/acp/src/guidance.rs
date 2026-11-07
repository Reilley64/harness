use application::use_cases::read_guidance;
use serde_json::json;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/guidance/read",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadRequest {
    pub cwd: std::path::PathBuf,
    pub session_id: Option<String>,
    pub phase: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub references: Option<Vec<String>>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method="_harness/guidance/adopt",response=crate::agents::Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AdoptRequest {
    pub session_id: String,
    pub revision: u64,
    pub available: String,
    pub phase: String,
    pub scopes: Vec<String>,
    pub references: Vec<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    pub revision: Option<u64>,
    pub selected: Option<serde_json::Value>,
    pub current: Option<serde_json::Value>,
    pub current_run: Option<String>,
    pub historical_unknown: Vec<String>,
    pub available: Option<serde_json::Value>,
    pub blocked: Vec<String>,
    pub overrides: Vec<String>,
    pub bundled: Vec<serde_json::Value>,
}

impl From<read_guidance::Output> for Response {
    fn from(output: read_guidance::Output) -> Self {
        Self {
            revision: output.revision,
            selected: output.selected.as_ref().map(snapshot),
            current: output.current.as_ref().map(snapshot),
            current_run: output.current_run,
            historical_unknown: output.historical_unknown,
            available: output.resolution.available.as_ref().map(snapshot),
            blocked: output.resolution.blocked,
            overrides: output.resolution.overrides,
            bundled: output.resolution.bundled.iter().map(instruction).collect(),
        }
    }
}

fn instruction(instruction: &domain::Instruction) -> serde_json::Value {
    json!({
        "id": instruction.id, "version": instruction.version,
        "source": instruction.source, "upstream": instruction.upstream,
        "requires": instruction.requires, "body": instruction.body,
    })
}

fn snapshot(snapshot: &domain::InstructionSnapshot) -> serde_json::Value {
    json!({
        "id": snapshot.id, "phase": snapshot.phase,
        "scopes": snapshot.scopes, "references": snapshot.references,
        "instructions": snapshot.instructions.iter().map(instruction).collect::<Vec<_>>(),
    })
}
