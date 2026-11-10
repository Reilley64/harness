use super::Dependencies;
use super::Error;
use crate::error_context::ErrorContext;
use crate::execution_ancestry;
use crate::ports::ReadCoordination;
use crate::ports::UpdateCoordination;
use crate::turn_control::SteeringRequest;
use crate::use_cases::read_children;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(super) struct Lease {
    controller: String,
    run_id: String,
}

pub(super) fn parent(conversation: &domain::Conversation) -> bool {
    conversation.agent.as_ref().is_some_and(|agent| {
        matches!(
            agent.role,
            domain::AgentRole::ChiefOfStaff
                | domain::AgentRole::Griller
                | domain::AgentRole::Research
        )
    })
}

pub(super) async fn claim(
    d: &Dependencies,
    conversation: &domain::Conversation,
    cancellation: &CancellationToken,
) -> Result<Option<Lease>, Error> {
    if !parent(conversation) {
        return Ok(None);
    }
    let run_id = conversation
        .runs
        .last()
        .ok_or_else(|| Error::caused_by("The parent has no active logical run"))?
        .id
        .clone();
    let (mut coordination, diagnostics) = read_children::load(
        &read_children::Dependencies {
            read_coordination: d.read_coordination.clone(),
            read_conversation: d.read_conversation.clone(),
            read_execution_claims: d.read_execution_claims.clone(),
            update_coordination: d.update_coordination.clone(),
        },
        conversation.id.clone(),
    )
    .await
    .map_err(Error::caused_by)?;
    if !diagnostics.is_empty() {
        return Err(Error::caused_by(read_children::retained(diagnostics)));
    }
    if coordination.as_ref().is_some_and(|state| {
        matches!(
            state.gate,
            domain::CoordinationGate::NeedsHuman
                | domain::CoordinationGate::Stopped
                | domain::CoordinationGate::Unknown
        ) && state.authority_complete
            && state.controller.is_none()
    }) {
        let response = d
            .update_coordination
            .call(
                conversation.id.clone(),
                domain::CoordinationCommand::Gate {
                    gate: domain::CoordinationGate::Open,
                    controller: None,
                    run_id: None,
                },
                coordination.as_ref().map(|state| state.revision),
                cancellation.clone(),
            )
            .await
            .map_err(Error::caused_by)?;
        coordination = Some(response.coordination);
    }
    if coordination.is_none()
        && conversation
            .messages
            .iter()
            .flat_map(|message| &message.steps)
            .flat_map(|step| &step.calls)
            .any(|call| {
                call.name == domain::Tool::DispatchResearch.as_str()
                    && !matches!(
                        call.status,
                        domain::ToolStatus::Draft
                            | domain::ToolStatus::Requested
                            | domain::ToolStatus::Invalid
                            | domain::ToolStatus::Skipped
                    )
            })
    {
        return Err(Error::caused_by(ErrorContext::caused_by(
            "Required child coordination is missing. Retain and inspect the durable execution claims before continuing; no empty child view was substituted.",
            "Conversation records an admitted managed dispatch without its coordination authority",
        )));
    }
    let controller = coordination
        .as_ref()
        .filter(|state| state.run_id.as_deref() == Some(&run_id))
        .and_then(|state| state.controller.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    d.update_coordination
        .call(
            conversation.id.clone(),
            domain::CoordinationCommand::Acquire {
                agent_id: conversation
                    .agent
                    .as_ref()
                    .ok_or_else(|| Error::caused_by("Parent agent identity is missing"))?
                    .id
                    .0
                    .clone(),
                controller: controller.clone(),
                run_id: run_id.clone(),
            },
            coordination.map(|state| state.revision),
            cancellation.clone(),
        )
        .await
        .map_err(Error::caused_by)?;
    Ok(Some(Lease { controller, run_id }))
}

pub(super) async fn disposition(
    d: &Dependencies,
    conversation: &domain::Conversation,
) -> Result<domain::CoordinationGate, Error> {
    let mut gate = domain::CoordinationGate::Open;
    if let Some(state) = d
        .read_coordination
        .call(conversation.id.clone())
        .await
        .map_err(Error::caused_by)?
    {
        if conversation
            .agent
            .as_ref()
            .is_none_or(|agent| agent.id.0 != state.agent_id)
            || state
                .run_id
                .as_ref()
                .is_some_and(|id| conversation.runs.last().is_none_or(|run| run.id != *id))
        {
            return Err(Error::caused_by(
                "Stop disposition ownership differs from its durable run",
            ));
        }
        if state.gate == domain::CoordinationGate::Stopped {
            return Ok(domain::CoordinationGate::Stopped);
        }
        gate = state.gate;
    }
    if let Some(research) = &conversation.research {
        let ancestor = execution_ancestry::gate(
            &d.read_coordination,
            &d.read_conversation,
            &conversation.id,
            research,
        )
        .await
        .map_err(Error::caused_by)?;
        if matches!(
            ancestor,
            domain::CoordinationGate::Stopped | domain::CoordinationGate::ResumeRequired
        ) {
            return Ok(ancestor);
        }
    }
    Ok(gate)
}

pub(super) async fn monitor(
    read: &ReadCoordination,
    conversation_id: String,
    run_id: String,
) -> Result<(), Error> {
    let mut admitted = false;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        let state = read
            .call(conversation_id.clone())
            .await
            .map_err(Error::caused_by)?;
        match state {
            Some(state) if state.run_id.as_deref() == Some(run_id.as_str()) => {
                admitted = true;
                if state.gate == domain::CoordinationGate::Stopped {
                    return Ok(());
                }
                if state.gate != domain::CoordinationGate::Open {
                    return Err(Error::caused_by(
                        "Execution gate closed; results remain retained without continuation",
                    ));
                }
            }
            _ if admitted => {
                return Err(Error::caused_by(
                    "Controlling execution disappeared or changed; inspect without replay",
                ));
            }
            _ => {}
        }
    }
}

pub(super) async fn admit(
    d: &Dependencies,
    conversation_id: &str,
    lease: Option<&Lease>,
) -> Result<(), Error> {
    let Some(lease) = lease else { return Ok(()) };
    let state = d
        .read_coordination
        .call(conversation_id.into())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| {
            Error::caused_by("Required execution coordination is missing before tool admission")
        })?;
    if state.gate != domain::CoordinationGate::Open
        || state.controller.as_deref() != Some(lease.controller.as_str())
        || state.run_id.as_deref() != Some(lease.run_id.as_str())
        || !state.authority_complete
    {
        return Err(Error::caused_by(
            "Execution is stopped, gated or has a different controller; no new tool admitted",
        ));
    }
    Ok(())
}

async fn transition(
    update: &UpdateCoordination,
    state: &domain::Coordination,
    command: domain::CoordinationCommand,
    cancellation: &CancellationToken,
) -> Result<domain::Coordination, Error> {
    Ok(update
        .call(
            state.conversation_id.clone(),
            command,
            Some(state.revision),
            cancellation.clone(),
        )
        .await
        .map_err(Error::caused_by)?
        .coordination)
}

pub(super) async fn prepare(
    d: &Dependencies,
    conversation: &mut domain::Conversation,
    lease: Option<&Lease>,
    diagnostics: &mut Vec<Box<dyn std::error::Error + Send + Sync>>,
    cancellation: &CancellationToken,
) -> Result<(), Error> {
    let Some(lease) = lease else { return Ok(()) };
    for _ in 0..8 {
        let (state, recovered) = read_children::load(
            &read_children::Dependencies {
                read_coordination: d.read_coordination.clone(),
                read_conversation: d.read_conversation.clone(),
                read_execution_claims: d.read_execution_claims.clone(),
                update_coordination: d.update_coordination.clone(),
            },
            conversation.id.clone(),
        )
        .await
        .map_err(Error::caused_by)?;
        diagnostics.extend(recovered);
        let state = state.ok_or_else(|| {
            Error::caused_by("Required parent coordination is missing; inspect without inference")
        })?;
        if !state.eligible(&lease.run_id).map_err(Error::caused_by)? {
            return Err(Error::caused_by(
                "Parent continuation is gated or a captured join is pending",
            ));
        }
        let command = domain::CoordinationCommand::Deliver {
            controller: lease.controller.clone(),
            run_id: lease.run_id.clone(),
            outcomes: state
                .children
                .iter()
                .flat_map(|child| child.receipts())
                .filter(|(_, delivered, _)| delivered.is_none())
                .map(|(outcome, _, _)| outcome.id.clone())
                .collect(),
        };
        let (revision, command, view) = tokio::task::spawn_blocking(move || {
            let mut admitted = state.clone();
            if admitted.apply(command.clone()).map_err(Error::caused_by)? {
                admitted.revision = admitted
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| Error::caused_by("Coordination revision exhausted"))?;
            }
            let view = read_children::project(&admitted).map_err(|error|Error::caused_by(ErrorContext::caused_by("Required current-work facts cannot fit the 64 KiB context. Inspect retained source-linked results and joins before continuing; no facts were truncated.",error)))?;
            Ok::<_, Error>((state.revision, command, view))
        })
        .await
        .map_err(Error::caused_by)??;
        match d
            .update_coordination
            .call(
                conversation.id.clone(),
                command,
                Some(revision),
                cancellation.clone(),
            )
            .await
        {
            Ok(_) => {
                conversation.current_work = Some(view);
                return Ok(());
            }
            Err(error) if error.conflict => diagnostics.push(Box::new(error)),
            Err(error) => return Err(Error::caused_by(error)),
        }
    }
    Err(Error::caused_by(
        "Current-work admission kept changing; inspect retained outcomes before continuing",
    ))
}

pub(super) async fn wait(
    d: &Dependencies,
    conversation: &mut domain::Conversation,
    lease: Option<&Lease>,
    after_response: bool,
    buffer: &Arc<Mutex<String>>,
    steering: &mut tokio::sync::mpsc::Receiver<SteeringRequest>,
    cancellation: &CancellationToken,
) -> Result<bool, Error> {
    let Some(lease) = lease else { return Ok(false) };
    loop {
        if cancellation.is_cancelled() {
            return Ok(false);
        }
        super::instructions::wait(d, conversation, cancellation).await?;
        let newly_steered =
            super::instructions::synchronize(d, conversation, buffer, cancellation).await?;
        let state = d
            .read_coordination
            .call(conversation.id.clone())
            .await
            .map_err(Error::caused_by)?
            .ok_or_else(|| {
                Error::caused_by("Required parent coordination is missing; inspect without replay")
            })?;
        if state.gate != domain::CoordinationGate::Open {
            return Err(Error::caused_by(
                "Parent continuation is gated; outcomes remain retained without inference",
            ));
        }
        let eligible = state.eligible(&lease.run_id).map_err(Error::caused_by)?;
        if after_response && eligible && newly_steered {
            return Ok(true);
        }
        if !after_response && eligible {
            return Ok(false);
        }
        let required = state.children.iter().any(|child| {
            child.outcome.is_none()
                || child
                    .receipts()
                    .any(|(_, _, completed)| completed.is_none())
        });
        if after_response && !required {
            return Ok(false);
        }
        if eligible
            && state.children.iter().any(|child| {
                child
                    .receipts()
                    .any(|(_, delivered, _)| delivered.is_none())
            })
        {
            return Ok(true);
        }
        if after_response
            && state.children.iter().any(|child| {
                child
                    .receipts()
                    .any(|(_, delivered, completed)| delivered.is_some() && completed.is_none())
            })
        {
            return Ok(false);
        }
        if after_response
            && state.children.iter().any(|child| {
                child.status == domain::ChildExecutionStatus::Unknown && child.outcome.is_none()
            })
        {
            return Ok(false);
        }
        if !state.waiting {
            transition(
                &d.update_coordination,
                &state,
                domain::CoordinationCommand::Wait {
                    controller: lease.controller.clone(),
                    run_id: lease.run_id.clone(),
                },
                cancellation,
            )
            .await?;
        }
        tokio::select! {
            _ = cancellation.cancelled() => return Ok(false),
            Some(request) = steering.recv() => {
                if super::accept(d, conversation, request, buffer, cancellation).await? && eligible {
                    return Ok(true);
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
        }
    }
}

pub(super) async fn release(
    d: &Dependencies,
    conversation: &domain::Conversation,
    status: domain::MessageStatus,
) -> Result<(), Error> {
    if !parent(conversation) {
        return Ok(());
    }
    let Some(state) = d
        .read_coordination
        .call(conversation.id.clone())
        .await
        .map_err(Error::caused_by)?
    else {
        return Ok(());
    };
    let Some(controller) = state.controller.clone() else {
        return Ok(());
    };
    let Some(run_id) = state.run_id.clone() else {
        return Err(Error::caused_by("Controlling execution has no logical run"));
    };
    if conversation.runs.last().is_none_or(|run| run.id != run_id) {
        return Err(Error::caused_by(
            "Controlling execution belongs to another run; inspect without replay",
        ));
    }
    let gate = match status {
        domain::MessageStatus::Cancelled => domain::CoordinationGate::Stopped,
        domain::MessageStatus::Failed | domain::MessageStatus::Interrupted => {
            domain::CoordinationGate::Unknown
        }
        _ => domain::CoordinationGate::NeedsHuman,
    };
    let token = CancellationToken::new();
    let state = transition(
        &d.update_coordination,
        &state,
        domain::CoordinationCommand::Gate {
            gate,
            controller: Some(controller.clone()),
            run_id: Some(run_id.clone()),
        },
        &token,
    )
    .await?;
    transition(
        &d.update_coordination,
        &state,
        domain::CoordinationCommand::Release { controller, run_id },
        &token,
    )
    .await?;
    Ok(())
}
