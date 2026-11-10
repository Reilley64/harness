use super::Dependencies;
use super::Error;
use crate::execution_ancestry;
use crate::turn_control::SteeringRequest;
use parking_lot::Mutex;
use std::fmt::Write;
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
    let boundaries = execution_ancestry::read(
        &d.read_coordination,
        &d.read_conversation,
        &conversation.id,
        &research,
    )
    .await
    .map_err(Error::caused_by)?;
    if boundaries.iter().any(|boundary| {
        boundary
            .scope
            .as_ref()
            .is_some_and(|scope| scope.approved_by.is_none())
    }) {
        return Ok(true);
    }
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

pub(super) async fn wait(
    d: &Dependencies,
    conversation: &domain::Conversation,
    cancellation: &CancellationToken,
) -> Result<(), Error> {
    let Some(research) = conversation.research.as_ref() else {
        return Ok(());
    };
    loop {
        if cancellation.is_cancelled() {
            return Ok(());
        }
        let boundaries = execution_ancestry::read(
            &d.read_coordination,
            &d.read_conversation,
            &conversation.id,
            research,
        )
        .await
        .map_err(Error::caused_by)?;
        if !boundaries.iter().any(|boundary| {
            boundary
                .scope
                .as_ref()
                .is_some_and(|scope| scope.approved_by.is_none())
        }) && !boundaries
            .iter()
            .skip(1)
            .any(|boundary| boundary.pending_instructions)
        {
            return Ok(());
        }
        tokio::select! {
            _ = cancellation.cancelled() => return Ok(()),
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
        }
    }
}

pub(super) async fn project(
    d: &Dependencies,
    conversation: &mut domain::Conversation,
) -> Result<(), Error> {
    let Some(research) = &conversation.research else {
        return Ok(());
    };
    let boundaries = execution_ancestry::read(
        &d.read_coordination,
        &d.read_conversation,
        &conversation.id,
        research,
    )
    .await
    .map_err(Error::caused_by)?;
    let mut view = conversation.current_work.take().unwrap_or_default();
    for boundary in boundaries {
        if let Some(scope) = boundary.scope {
            if scope.approved_by.is_none() {
                return Err(Error::caused_by(
                    "Scope approval closed before model context admission",
                ));
            }
            writeln!(&mut view, "TRUSTED HUMAN SCOPE: execution {} request {} exact version {} approved by {:?}. Source harness://coordination/{}. Criteria: {:?}", boundary.execution_id, scope.id, scope.version, scope.approved_by, boundary.parent_id, scope.text).map_err(Error::caused_by)?;
            if view.len() > 65_536 {
                return Err(Error::caused_by(
                    "Required scope authority exceeds current-work capacity; no criteria were truncated",
                ));
            }
        }
    }
    conversation.current_work = Some(view);
    Ok(())
}

pub(super) async fn admit(
    d: &Dependencies,
    id: &str,
    research: Option<&domain::ResearchAssignment>,
    diagnostics: &mut Vec<Box<dyn std::error::Error + Send + Sync>>,
    cancellation: &CancellationToken,
) -> Result<bool, Error> {
    let Some(research) = research else {
        return Ok(true);
    };
    let boundaries =
        execution_ancestry::read(&d.read_coordination, &d.read_conversation, id, research)
            .await
            .map_err(Error::caused_by)?;
    if boundaries.iter().any(|boundary| {
        boundary.pending_instructions
            || boundary
                .scope
                .as_ref()
                .is_some_and(|scope| scope.approved_by.is_none())
    }) {
        return Ok(false);
    }
    for (index, boundary) in boundaries.into_iter().enumerate() {
        let command = if index == 0 {
            domain::CoordinationCommand::AdmitChildAction {
                execution_id: boundary.execution_id,
                controller: boundary.controller,
            }
        } else {
            domain::CoordinationCommand::AdmitDescendantAction {
                execution_id: boundary.execution_id,
                controller: boundary.controller,
            }
        };
        if let Err(original) = d
            .update_coordination
            .call(boundary.parent_id, command, None, cancellation.clone())
            .await
        {
            match execution_ancestry::read(&d.read_coordination, &d.read_conversation, id, research)
                .await
            {
                Ok(fresh)
                    if fresh.iter().any(|boundary| {
                        boundary.pending_instructions
                            || boundary
                                .scope
                                .as_ref()
                                .is_some_and(|scope| scope.approved_by.is_none())
                    }) =>
                {
                    diagnostics.push(Box::new(original));
                    return Ok(false);
                }
                Ok(_) => return Err(Error::caused_by(original)),
                Err(read) => {
                    return Err(super::error::retain(Error::caused_by(original), vec![read]));
                }
            }
        }
    }
    Ok(true)
}
