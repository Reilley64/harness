use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    path: Option<String>,
    limit: Option<usize>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid ls arguments")?;
    Ok(domain::ToolInvocation::ReadDirectory(
        domain::ReadDirectoryArguments {
            path: value.path,
            limit: value.limit,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"ls","description":"List direct directory entries, including hidden entries.","strict":true,"parameters":{"type":"object","properties":{"path":{"type":["string","null"]},"limit":{"type":["integer","null"]}},"required":["path","limit"],"additionalProperties":false}})
}
