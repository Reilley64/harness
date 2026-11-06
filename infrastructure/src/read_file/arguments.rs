use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid read arguments")?;
    Ok(domain::ToolInvocation::ReadFile(
        domain::ReadFileArguments {
            path: value.path,
            offset: value.offset,
            limit: value.limit,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"read","description":"Read a UTF-8 file with 1-based line offset and bounded output. Absolute paths are allowed.","strict":true,"parameters":{"type":"object","properties":{"path":{"type":"string"},"offset":{"type":["integer","null"]},"limit":{"type":["integer","null"]}},"required":["path","offset","limit"],"additionalProperties":false}})
}
