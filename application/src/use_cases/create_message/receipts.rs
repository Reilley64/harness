use super::Dependencies;
use super::Error;
use super::error;
use tokio_util::sync::CancellationToken;

pub(super) async fn persist(
    d: &Dependencies,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    if conversation.resume_required {
        return Ok(());
    }
    let research = conversation
        .research
        .as_ref()
        .ok_or_else(|| Error::caused_by("Recovered child has no execution owner"))?;
    let state = d
        .read_coordination
        .call(research.parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Recovered child ownership is missing"))?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == research.execution_id)
        .ok_or_else(|| Error::caused_by("Recovered child execution is missing"))?;
    let reply = conversation
        .messages
        .last()
        .ok_or_else(|| Error::caused_by("Recovered child has no reply"))?;
    let run = conversation
        .runs
        .last()
        .ok_or_else(|| Error::caused_by("Recovered child has no run"))?;
    let uncertain = reply
        .steps
        .iter()
        .flat_map(|step| &step.calls)
        .any(|call| call.status == domain::ToolStatus::Unknown);
    let unsettled = d
        .read_coordination
        .call(conversation.id.clone())
        .await
        .map_err(Error::caused_by)?
        .is_some_and(|state| !state.settled());
    let status = if uncertain {
        domain::ChildExecutionStatus::Unknown
    } else {
        match reply.status {
            domain::MessageStatus::Completed if !unsettled => {
                domain::ChildExecutionStatus::Completed
            }
            domain::MessageStatus::Cancelled => domain::ChildExecutionStatus::Stopped,
            domain::MessageStatus::Failed => domain::ChildExecutionStatus::Failed,
            _ => domain::ChildExecutionStatus::NeedsHuman,
        }
    };
    let outcome = domain::ChildOutcome {
        id: run.id.clone(),
        status,
        conversation_id: conversation.id.clone(),
        run_id: Some(run.id.clone()),
        reply_id: Some(reply.id.clone()),
        summary: if reply.content.len() <= 8192 {
            reply.content.clone()
        } else {
            "Recovered report exceeds its bounded summary; inspect its retained source conversation.".into()
        },
        artifacts: vec![format!("harness://conversations/{}", conversation.id)],
    };
    let received = d
        .update_coordination
        .call(
            research.parent_id.clone(),
            domain::CoordinationCommand::Receive {
                execution_id: research.execution_id.clone(),
                controller: child.controller.clone(),
                outcome,
            },
            None,
            CancellationToken::new(),
        )
        .await;
    if let Err(original) = received {
        let fenced = d
            .update_coordination
            .call(
                research.parent_id.clone(),
                domain::CoordinationCommand::MarkUnknown {
                    execution_id: research.execution_id.clone(),
                    controller: child.controller.clone(),
                },
                None,
                CancellationToken::new(),
            )
            .await;
        return Err(match fenced {
            Ok(_) => Error::caused_by(original),
            Err(fence) => error::retain(Error::caused_by(original), vec![Box::new(fence)]),
        });
    }
    Ok(())
}
