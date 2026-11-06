use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    pattern: String,
    path: Option<String>,
    limit: Option<usize>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid find arguments")?;
    Ok(domain::ToolInvocation::ReadPaths(
        domain::ReadPathsArguments {
            pattern: value.pattern,
            path: value.path,
            limit: value.limit,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"find","description":"Find paths matching a glob, respecting ignore files.","strict":true,"parameters":{"type":"object","properties":{"pattern":{"type":"string"},"path":{"type":["string","null"]},"limit":{"type":["integer","null"]}},"required":["pattern","path","limit"],"additionalProperties":false}})
}
