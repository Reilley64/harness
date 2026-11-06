use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    pattern: String,
    path: Option<String>,
    glob: Option<String>,
    literal: Option<bool>,
    ignore_case: Option<bool>,
    context_lines: Option<usize>,
    limit: Option<usize>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid grep arguments")?;
    Ok(domain::ToolInvocation::ReadTextMatches(
        domain::ReadTextMatchesArguments {
            pattern: value.pattern,
            path: value.path,
            glob: value.glob,
            literal: value.literal,
            ignore_case: value.ignore_case,
            context_lines: value.context_lines,
            limit: value.limit,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"grep","description":"Search text using Rust regex or literal matching, respecting ignore files.","strict":true,"parameters":{"type":"object","properties":{"pattern":{"type":"string"},"path":{"type":["string","null"]},"glob":{"type":["string","null"]},"literal":{"type":["boolean","null"]},"ignoreCase":{"type":["boolean","null"]},"contextLines":{"type":["integer","null"]},"limit":{"type":["integer","null"]}},"required":["pattern","path","glob","literal","ignoreCase","contextLines","limit"],"additionalProperties":false}})
}
