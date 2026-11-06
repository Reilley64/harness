use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    pattern: String,
    language: String,
    path: Option<String>,
    glob: Option<String>,
    limit: Option<usize>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid ast_search arguments")?;
    Ok(domain::ToolInvocation::ReadCodeMatches(
        domain::ReadCodeMatchesArguments {
            pattern: value.pattern,
            language: value.language,
            path: value.path,
            glob: value.glob,
            limit: value.limit,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"ast_search","description":"Read-only structural search with built-in ast-grep languages; not type-aware.","strict":true,"parameters":{"type":"object","properties":{"pattern":{"type":"string"},"language":{"type":"string","enum":super::LANGUAGES},"path":{"type":["string","null"]},"glob":{"type":["string","null"]},"limit":{"type":["integer","null"]}},"required":["pattern","language","path","glob","limit"],"additionalProperties":false}})
}
