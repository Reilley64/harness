use crate::tool_io;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    question: String,
}

pub(super) fn decode(arguments: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    let arguments: Arguments = serde_json::from_str(arguments)?;
    if arguments.question.trim().is_empty() || arguments.question.len() > 8192 {
        return Err("Human input requires a nonblank question of at most 8192 UTF-8 bytes".into());
    }
    Ok(domain::ToolInvocation::RequestHumanInput(
        arguments.question,
    ))
}

pub(crate) fn definition() -> serde_json::Value {
    serde_json::json!({
        "type": "function", "name": "request_human_input",
        "description": "End this run for an explicit human answer. Updates and child results cannot answer this request. The matching answer starts a later run on this same durable agent.",
        "parameters": {
            "type": "object", "properties": { "question": { "type": "string", "minLength": 1, "maxLength": 8192 } },
            "required": ["question"], "additionalProperties": false,
        },
    })
}
