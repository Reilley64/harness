use super::Error;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    id: String,
    mode: String,
    executions: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BackgroundArguments {
    id: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FollowUpArguments {
    outcomes: Vec<String>,
}

pub(crate) fn decode(tool: domain::Tool, value: &str) -> Result<domain::ToolInvocation, Error> {
    let control = match tool {
        domain::Tool::JoinChildren => {
            let arguments: Arguments = serde_json::from_str(value)?;
            domain::ChildrenControl::Join {
                id: arguments.id,
                mode: match arguments.mode.as_str() {
                    "any" => domain::JoinMode::Any,
                    "all" => domain::JoinMode::All,
                    _ => return Err("Join mode must be any or all".into()),
                },
                executions: arguments.executions,
            }
        }
        domain::Tool::BackgroundChildren => {
            let arguments: BackgroundArguments = serde_json::from_str(value)?;
            domain::ChildrenControl::Background { id: arguments.id }
        }
        domain::Tool::CompleteChildrenFollowUp => {
            let arguments: FollowUpArguments = serde_json::from_str(value)?;
            if arguments.outcomes.is_empty() || arguments.outcomes.len() > 256 {
                return Err("Follow-up must identify 1..256 delivered outcome receipts".into());
            }
            domain::ChildrenControl::CompleteFollowUp {
                outcomes: arguments.outcomes,
            }
        }
        _ => return Err("Unsupported child control".into()),
    };
    Ok(domain::ToolInvocation::ControlChildren(control))
}

pub(crate) fn definition(tool: domain::Tool) -> serde_json::Value {
    match tool {
        domain::Tool::JoinChildren => serde_json::json!({
            "type":"function", "name":"join_children",
            "description":"Capture a fixed set of accepted execution IDs and suspend this same logical run until any/all complete or partial failure/input needs reconciliation. The tool returns its durable acknowledgement before waiting; no model polling. Later children never enlarge this set.",
            "parameters":{"type":"object","properties":{"id":{"type":"string"},"mode":{"type":"string","enum":["any","all"]},"executions":{"type":"array","items":{"type":"string"},"minItems":1,"maxItems":256}},"required":["id","mode","executions"],"additionalProperties":false}
        }),
        domain::Tool::BackgroundChildren => serde_json::json!({
            "type":"function", "name":"background_children",
            "description":"Release the named captured join without restarting, cancelling or replacing its children. Required results/follow-up remain pending; a response is not settlement.",
            "parameters":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}
        }),
        domain::Tool::CompleteChildrenFollowUp => serde_json::json!({
            "type":"function", "name":"complete_children_followup",
            "description":"Record completion of required parent follow-up for these exact delivered outcome receipt IDs, after handling their report/effects/uncertainty. Receipt or a model response alone never completes follow-up. This cannot approve scope, close work, replay or authorize merge.",
            "parameters":{"type":"object","properties":{"outcomes":{"type":"array","items":{"type":"string"},"minItems":1,"maxItems":256}},"required":["outcomes"],"additionalProperties":false}
        }),
        _ => unreachable!(),
    }
}
