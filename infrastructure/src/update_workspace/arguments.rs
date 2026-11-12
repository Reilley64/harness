#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    commit: String,
}

pub(crate) fn decode(
    value: &str,
) -> Result<domain::ToolInvocation, Box<dyn std::error::Error + Send + Sync>> {
    let arguments: Arguments = serde_json::from_str(value)?;
    if arguments.commit.len() != 40 && arguments.commit.len() != 64
        || !arguments
            .commit
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("Submission requires an exact full native commit hash".into());
    }
    Ok(domain::ToolInvocation::SubmitImplementation(
        arguments.commit,
    ))
}

pub(crate) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"submit_implementation","description":"Submit this owned Implementer's exact full HEAD commit. Harness verifies native ownership, a clean worktree and all required checks, and persists check evidence before returning. A report or this submission is not review acceptance or merge authority.","parameters":{"type":"object","properties":{"commit":{"type":"string"}},"required":["commit"],"additionalProperties":false}})
}
