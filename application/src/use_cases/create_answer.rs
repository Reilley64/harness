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
    answer(
        dependencies.into(),
        id,
        request_run,
        None,
        content,
        present,
        steering,
        workspace,
        cancellation,
    )
    .await
}

pub(crate) async fn approved_scope(
    dependencies: Dependencies,
    id: String,
    scope_id: String,
    version: String,
    approval_id: String,
    workspace: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = &dependencies.worker;
    let conversation = d
        .read_conversation
        .call(id.clone())
        .await
        .map_err(Error::caused_by)?;
    let research = conversation
        .research
        .as_ref()
        .ok_or_else(|| Error::caused_by("Scope approval requires retained Research ownership"))?;
    let state = d
        .read_coordination
        .call(research.parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Scope approval lost its durable ownership"))?;
    let scope = state
        .scope(&research.execution_id)
        .filter(|scope| {
            scope.id == scope_id
                && scope.version == version
                && scope.approved_by.as_deref() == Some(approval_id.as_str())
        })
        .ok_or_else(|| Error::caused_by("Scope approval is superseded or no longer exact"))?;
    let request_run = scope.supersedes_run.clone().ok_or_else(|| {
        Error::caused_by("Scope approval cannot implicitly answer a human question")
    })?;
    let content = format!(
        "Trusted human approval {} explicitly superseded input run {} for exact assignment scope version {}. Scope request {}. New criteria: {}",
        approval_id, request_run, version, scope_id, scope.text
    );
    let (_sender, steering) = tokio::sync::mpsc::channel(32);
    answer(
        dependencies,
        id,
        request_run,
        Some(approval_id),
        content,
        PresentTurnEvent::new(|_| Ok(())),
        steering,
        workspace,
        cancellation,
    )
    .await
}

async fn answer(
    dependencies: Dependencies,
    id: String,
    request_run: String,
    approval_id: Option<String>,
    content: String,
    present: PresentTurnEvent,
    steering: tokio::sync::mpsc::Receiver<SteeringRequest>,
    workspace: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.worker;
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
            approval_id,
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
    if let Some(scope) = state.scope(&research.execution_id)
        && (scope.approved_by.is_none()
            || (scope.supersedes_run.as_deref() == Some(request_run.as_str())
                && scope.approved_by != approval_id))
    {
        return Err(Error::caused_by(
            "Only matching exact-version approval can supersede this input; an answer cannot clear scope approval",
        ));
    }
    let _owner = d
        .lock_execution
        .call(research.execution_id.clone())
        .await
        .map_err(Error::caused_by)?;
    let controller = child.controller.clone();
    let result = create_message::answer(
        d.clone(),
        id.clone(),
        request_run,
        approval_id,
        content,
        present,
        steering,
        workspace,
        cancellation,
    )
    .await;
    let mut output = match result {
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
    let retained = !output.tool_errors.is_empty();
    if retained {
        let run = output.conversation.runs.last().ok_or_else(|| {
            error::retain(
                Error::caused_by("Later diagnostics have no durable run"),
                std::mem::take(&mut output.tool_errors),
            )
        })?;
        d.create_execution_diagnostics
            .call(
                research.parent_id.clone(),
                research.execution_id.clone(),
                Some(run.id.clone()),
                std::mem::take(&mut output.tool_errors),
            )
            .await
            .map_err(Error::caused_by)?;
    }
    if output.conversation.resume_required {
        return Ok(Output {
            conversation: output.conversation,
            tool_errors: output.tool_errors,
        });
    }
    let unsettled = d
        .read_coordination
        .call(output.conversation.id.clone())
        .await
        .map_err(|error| {
            error::retain(
                Error::caused_by(error),
                std::mem::take(&mut output.tool_errors),
            )
        })?
        .is_some_and(|state| !state.settled());
    let execution_id = research.execution_id.clone();
    let (mut conversation, outcome) = tokio::task::spawn_blocking(move || {
        let conversation = output.conversation;
        let reply = conversation.messages.last().ok_or_else(|| Error::caused_by("Later child run has no reply"))?;
        let run = conversation.runs.last().ok_or_else(|| Error::caused_by("Later child run has no durable run"))?;
        let uncertain = reply.steps.iter().flat_map(|step| &step.calls).any(|call| call.status == domain::ToolStatus::Unknown);
        let status = if uncertain { domain::ChildExecutionStatus::Unknown } else {
            match reply.status {
                domain::MessageStatus::Completed if !unsettled => domain::ChildExecutionStatus::Completed,
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
            artifacts: if retained { vec![format!("harness://conversations/{id}"), format!("harness://execution-diagnostics/{}/runs/{}", execution_id, run.id)] } else { vec![format!("harness://conversations/{id}")] },
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
