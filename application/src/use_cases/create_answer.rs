mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::ports::PresentTurnEvent;
use crate::turn_control::SteeringRequest;
use crate::use_cases::create_message;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    request_run: String,
    content: String,
    present: PresentTurnEvent,
    steering: tokio::sync::mpsc::Receiver<SteeringRequest>,
    workspace: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into().worker;
    let conversation = d
        .read_conversation
        .call(id.clone())
        .await
        .map_err(Error::caused_by)?;
    let Some(research) = conversation.research else {
        let output = create_message::answer(
            d,
            id,
            request_run,
            content,
            present,
            steering,
            workspace,
            cancellation,
        )
        .await
        .map_err(Error::caused_by)?;
        return Ok(Output {
            conversation: output.conversation,
            tool_errors: output.tool_errors,
        });
    };
    let state = d
        .read_coordination
        .call(research.parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Human input has no durable execution owner"))?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == research.execution_id)
        .ok_or_else(|| Error::caused_by("Human input execution is missing"))?;
    if child.status != domain::ChildExecutionStatus::NeedsHuman
        || child
            .outcome
            .as_ref()
            .is_none_or(|outcome| outcome.run_id.as_deref() != Some(request_run.as_str()))
    {
        return Err(Error::caused_by(
            "Answer does not match the retained human-input receipt",
        ));
    }
    let controller = child.controller.clone();
    let result = create_message::answer(
        d.clone(),
        id.clone(),
        request_run,
        content,
        present,
        steering,
        workspace,
        cancellation,
    )
    .await;
    let output = match result {
        Ok(output) => output,
        Err(original) => {
            let _guard = match d.lock_conversation.call(id.clone()).await {
                Ok(guard) => guard,
                Err(lock) => {
                    return Err(error::retain(
                        Error::caused_by(original),
                        vec![Box::new(lock)],
                    ));
                }
            };
            let fence = d
                .update_coordination
                .call(
                    research.parent_id,
                    domain::CoordinationCommand::MarkUnknown {
                        execution_id: research.execution_id,
                        controller,
                    },
                    None,
                    CancellationToken::new(),
                )
                .await;
            return Err(match fence {
                Ok(_) => Error::caused_by(original),
                Err(fence) => error::retain(Error::caused_by(original), vec![Box::new(fence)]),
            });
        }
    };
    let (mut conversation, outcome) = tokio::task::spawn_blocking(move || {
        let conversation = output.conversation;
        let reply = conversation.messages.last().ok_or_else(|| Error::caused_by("Later child run has no reply"))?;
        let run = conversation.runs.last().ok_or_else(|| Error::caused_by("Later child run has no durable run"))?;
        let uncertain = reply.steps.iter().flat_map(|step| &step.calls).any(|call| call.status == domain::ToolStatus::Unknown);
        let status = if uncertain { domain::ChildExecutionStatus::Unknown } else {
            match reply.status {
                domain::MessageStatus::Completed => domain::ChildExecutionStatus::Completed,
                domain::MessageStatus::Cancelled => domain::ChildExecutionStatus::Stopped,
                domain::MessageStatus::Failed => domain::ChildExecutionStatus::Failed,
                _ => domain::ChildExecutionStatus::NeedsHuman,
            }
        };
        let outcome = domain::ChildOutcome {
            id: run.id.clone(), status, conversation_id: id.clone(),
            run_id: Some(run.id.clone()), reply_id: Some(reply.id.clone()),
            summary: if reply.content.len() <= 8192 { reply.content.clone() } else {
                "Later-run report exceeds its bounded summary; inspect the retained source conversation.".into()
            },
            artifacts: vec![format!("harness://conversations/{id}")],
        };
        Ok::<_, Error>((Output { conversation, tool_errors: output.tool_errors }, outcome))
    }).await.map_err(Error::caused_by)??;
    d.update_coordination
        .call(
            research.parent_id,
            domain::CoordinationCommand::Receive {
                execution_id: research.execution_id,
                controller,
                outcome,
            },
            None,
            CancellationToken::new(),
        )
        .await
        .map_err(|failure| {
            error::retain(
                Error::caused_by(failure),
                std::mem::take(&mut conversation.tool_errors),
            )
        })?;
    Ok(conversation)
}
