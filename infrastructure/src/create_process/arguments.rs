use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    command: String,
    timeout_seconds: Option<u64>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid bash arguments")?;
    Ok(domain::ToolInvocation::CreateProcess(
        domain::CreateProcessArguments {
            command: value.command,
            timeout_seconds: value.timeout_seconds,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"bash","description":"Run a noninteractive bash command with independent shell state and the OS user's permissions.","strict":true,"parameters":{"type":"object","properties":{"command":{"type":"string"},"timeoutSeconds":{"type":["integer","null"]}},"required":["command","timeoutSeconds"],"additionalProperties":false}})
}
