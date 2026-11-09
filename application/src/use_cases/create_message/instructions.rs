use super::Dependencies;
use super::Error;
use crate::turn_control::SteeringRequest;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(super) async fn synchronize(
    d: &Dependencies,
    conversation: &mut domain::Conversation,
    buffer: &Arc<Mutex<String>>,
    cancellation: &CancellationToken,
) -> Result<bool, Error> {
    let Some(research) = conversation.research.clone() else {
        return Ok(false);
    };
    let state = d
        .read_coordination
        .call(research.parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| {
            Error::caused_by("Instruction ownership is missing; no child action admitted")
        })?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == research.execution_id)
        .ok_or_else(|| Error::caused_by("Instruction execution owner is missing"))?;
    let mut changed = false;
    for instruction in state.instructions.iter().filter(|instruction| {
        instruction.execution_id == research.execution_id && instruction.applied_run.is_none()
    }) {
        let (accepted, acknowledgement) = tokio::sync::oneshot::channel();
        let applied = super::accept(
            d,
            conversation,
            SteeringRequest {
                id: instruction.id.clone(),
                text: instruction.text.clone(),
                accepted,
            },
            buffer,
            cancellation,
        )
        .await?;
        if acknowledgement.await != Ok(true) {
            return Err(Error::caused_by(
                "Durable human instruction could not be checkpointed; inspect before continuation",
            ));
        }
        changed |= applied;
        d.update_coordination
            .call(
                research.parent_id.clone(),
                domain::CoordinationCommand::ApplyChildInstruction {
                    id: instruction.id.clone(),
                    controller: child.controller.clone(),
                    run_id: conversation
                        .runs
                        .last()
                        .ok_or_else(|| Error::caused_by("Instruction has no active run"))?
                        .id
                        .clone(),
                    reply_id: conversation
                        .active_reply()
                        .map_err(Error::caused_by)?
                        .id
                        .clone(),
                },
                None,
                cancellation.clone(),
            )
            .await
            .map_err(Error::caused_by)?;
    }
    Ok(changed)
}

pub(super) async fn admit(
    d: &Dependencies,
    research: Option<&domain::ResearchAssignment>,
    cancellation: &CancellationToken,
) -> Result<(), Error> {
    let Some(research) = research else {
        return Ok(());
    };
    let state = d
        .read_coordination
        .call(research.parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Child action has no durable ownership"))?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == research.execution_id)
        .ok_or_else(|| Error::caused_by("Child action execution is missing"))?;
    d.update_coordination
        .call(
            research.parent_id.clone(),
            domain::CoordinationCommand::AdmitChildAction {
                execution_id: research.execution_id.clone(),
                controller: child.controller.clone(),
            },
            None,
            cancellation.clone(),
        )
        .await
        .map_err(Error::caused_by)?;
    Ok(())
}
