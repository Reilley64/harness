mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    parent_id: String,
    execution_id: String,
    id: String,
    text: String,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    for value in [&parent_id, &execution_id, &id] {
        if !uuid::Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == *value) {
            return Err(Error::caused_by(
                "Instruction identities must be canonical UUIDs",
            ));
        }
    }
    let state = d
        .read_coordination
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| {
            Error::caused_by("Targeted instruction requires retained execution ownership")
        })?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == execution_id)
        .ok_or_else(|| Error::caused_by("Targeted child is not owned by this parent"))?;
    if !state.authority_complete {
        return Err(Error::caused_by(
            "Instruction acknowledgement requires complete execution ownership",
        ));
    }
    if let Some(instruction) = state
        .instructions
        .iter()
        .find(|instruction| instruction.id == id)
    {
        return if instruction.execution_id == execution_id && instruction.text == text {
            Ok(Output {
                instruction: instruction.clone(),
            })
        } else {
            Err(Error::caused_by(
                "Instruction identity is already bound to a different exact payload",
            ))
        };
    }

    let conversation = d
        .read_conversation
        .call(child.agent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    if conversation.resume_required
        || conversation.active_reply().is_err()
        || conversation.active_reply().is_ok_and(|reply| {
            reply
                .generation_usage
                .as_ref()
                .is_some_and(|usage| usage.len() >= domain::MAX_MODEL_STEPS)
        })
    {
        return Err(Error::caused_by(
            "Use the matching answer, approval or Resume control; ordinary steering cannot answer a gate",
        ));
    }
    let response = d
        .update_coordination
        .call(
            parent_id,
            domain::CoordinationCommand::SteerChild {
                id: id.clone(),
                execution_id,
                text,
            },
            Some(state.revision),
            cancellation,
        )
        .await
        .map_err(Error::caused_by)?;
    let instruction = response
        .coordination
        .instructions
        .iter()
        .find(|instruction| instruction.id == id)
        .cloned()
        .ok_or_else(|| Error::caused_by("Durable instruction acknowledgement is missing"))?;
    Ok(Output { instruction })
}
