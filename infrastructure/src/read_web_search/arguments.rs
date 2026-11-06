use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    query: String,
    limit: Option<usize>,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid web_search arguments")?;
    Ok(domain::ToolInvocation::ReadWebSearch(
        domain::ReadWebSearchArguments {
            query: value.query,
            limit: value.limit,
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"web_search","description":"Search DuckDuckGo for titles, URLs, and snippets. Does not fetch result pages.","strict":true,"parameters":{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":["integer","null"]}},"required":["query","limit"],"additionalProperties":false}})
}
