use crate::tool_io;

use application::error_context::ResultExt;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    path: String,
    edits: Vec<Replacement>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Replacement {
    old_text: String,
    new_text: String,
}

pub(super) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let value: Arguments = serde_json::from_str(json).context("Invalid edit arguments")?;
    Ok(domain::ToolInvocation::UpdateFile(
        domain::UpdateFileArguments {
            path: value.path,
            edits: value
                .edits
                .into_iter()
                .map(|edit| domain::TextEdit {
                    old_text: edit.old_text,
                    new_text: edit.new_text,
                })
                .collect(),
        },
    ))
}
pub(super) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"edit","description":"Atomically apply exact unique non-overlapping text replacements.","strict":true,"parameters":{"type":"object","properties":{"path":{"type":"string"},"edits":{"type":"array","minItems":1,"maxItems":32,"items":{"type":"object","properties":{"oldText":{"type":"string"},"newText":{"type":"string"}},"required":["oldText","newText"],"additionalProperties":false}}},"required":["path","edits"],"additionalProperties":false}})
}
