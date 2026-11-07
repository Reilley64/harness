use crate::error_context;
use crate::ports::CreateModelStep;
mod dependencies;
mod error;
mod output;

use crate::model_access;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::turn_control::SteeringRequest;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::ports::CreateModelStepError;
use crate::ports::PresentTurnEvent;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    content: String,
    selection: Option<domain::ModelSelection>,
    present_turn_event: PresentTurnEvent,
    mut steering: tokio::sync::mpsc::Receiver<SteeringRequest>,
    workspace: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    if content.trim().is_empty() || content.len() > 1_048_576 {
        return Err(Error::caused_by(
            "Messages must contain text and be at most 1 MiB",
        ));
    }
    let _guard = dependencies
        .lock_conversation
        .call(id.clone())
        .await
        .map_err(Error::caused_by)?;
    let mut conversation = dependencies
        .read_conversation
        .call(id)
        .await
        .map_err(Error::caused_by)?;
    conversation.recover();
    if conversation.workspace.is_none() {
        conversation.workspace = Some(workspace);
    }
    let selection = selection.unwrap_or_else(|| domain::ModelSelection {
        provider_id: conversation.model.provider_id.clone(),
        model_id: conversation.model.id.clone(),
        effort: conversation.effort,
    });
    if selection.provider_id != conversation.model.provider_id {
        return Err(Error::caused_by(
            "Changing providers requires a new conversation",
        ));
    }
    conversation.model = model_access::read_model(
        &dependencies.read_models,
        &selection.provider_id,
        &selection.model_id,
        selection.effort,
    )
    .await
    .map_err(Error::caused_by)?;
    conversation.effort = selection.effort;
    model_access::read(
        &dependencies.read_model_allowlist,
        &dependencies.read_provider_connection,
        &dependencies.create_model_step_factory,
        &conversation.model,
    )
    .await
    .map_err(Error::caused_by)?;
    let create_model_step = dependencies
        .create_model_step_factory
        .call(conversation.model.provider_id.clone())
        .map_err(Error::caused_by)?;
    if cancellation.is_cancelled() {
        return Ok(Output { conversation });
    }
    let selected_model =
        domain::SelectedModel::new(conversation.model.clone(), conversation.effort)
            .map_err(Error::caused_by)?;
    dependencies
        .update_model_selection
        .call(selected_model)
        .await
        .map_err(Error::caused_by)?;
    if cancellation.is_cancelled() {
        return Ok(Output { conversation });
    }
    conversation
        .begin_message(
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
            content,
        )
        .map_err(Error::caused_by)?;
    conversation = dependencies
        .update_conversation
        .call(conversation)
        .await
        .map_err(Error::caused_by)?;
    let buffer = Arc::new(Mutex::new(String::new()));
    let streamed = buffer.clone();
    let visible = present_turn_event.clone();
    let present = PresentTurnEvent::new(move |event| {
        if let domain::TurnEvent::Text { delta, .. } = &event {
            let mut text = streamed.lock();
            if text.len().saturating_add(delta.len()) > 1_048_576 {
                return Err(CreateModelStepError::from(Box::<
                    dyn std::error::Error + Send + Sync,
                >::from(
                    "Reply exceeds 1 MiB"
                )));
            }
            text.push_str(delta);
        }
        visible.call(event)
    });
    let turn_cancellation = cancellation.child_token();
    let result = {
        let result = run(
            &dependencies,
            &create_model_step,
            &mut conversation,
            &buffer,
            &present,
            &mut steering,
            &turn_cancellation,
        );
        tokio::pin!(result);
        tokio::select! {outcome=&mut result=>outcome,_=tokio::time::sleep(std::time::Duration::from_secs(1200))=>{turn_cancellation.cancel();let _=result.await;Err(Error::caused_by("Turn exceeded 20 minutes"))}}
    };
    let status = if cancellation.is_cancelled() {
        domain::MessageStatus::Cancelled
    } else {
        match &result {
            Ok(status) => *status,
            Err(_) => domain::MessageStatus::Failed,
        }
    };
    conversation
        .finish_reply(buffer.lock().clone(), status)
        .map_err(Error::caused_by)?;
    checkpoint(&dependencies, &mut conversation).await?;
    if status != domain::MessageStatus::Cancelled {
        result?;
    }
    Ok(Output { conversation })
}

async fn checkpoint(
    dependencies: &Dependencies,
    conversation: &mut domain::Conversation,
) -> Result<(), Error> {
    *conversation = dependencies
        .update_conversation
        .call(conversation.clone())
        .await
        .map_err(Error::caused_by)?;
    Ok(())
}
async fn accept(
    dependencies: &Dependencies,
    conversation: &mut domain::Conversation,
    request: SteeringRequest,
    buffer: &Arc<Mutex<String>>,
    cancellation: &CancellationToken,
) -> Result<bool, Error> {
    if request.text.trim().is_empty()
        || request.text.len() > 1_048_576
        || !uuid::Uuid::parse_str(&request.id).is_ok_and(|id| id.to_string() == request.id)
    {
        let _ = request.accepted.send(false);
        return Ok(false);
    }
    match conversation.steering_accepted(&request.id, &request.text) {
        Ok(true) => {
            let _ = request.accepted.send(true);
            return Ok(false);
        }
        Err(_) => {
            let _ = request.accepted.send(false);
            return Ok(false);
        }
        Ok(false) => {}
    }
    if cancellation.is_cancelled() {
        let _ = request.accepted.send(false);
        return Ok(false);
    }
    conversation.active_reply().map_err(Error::caused_by)?;
    let changed = match conversation.steer(request.id, request.text, buffer.lock().clone()) {
        Ok(changed) => changed,
        Err(_) => {
            let _ = request.accepted.send(false);
            return Ok(false);
        }
    };
    if changed {
        checkpoint(dependencies, conversation).await?;
    }
    let _ = request.accepted.send(true);
    Ok(changed)
}
async fn run(
    dependencies: &Dependencies,
    create: &CreateModelStep,
    conversation: &mut domain::Conversation,
    buffer: &Arc<Mutex<String>>,
    present: &PresentTurnEvent,
    steering: &mut tokio::sync::mpsc::Receiver<SteeringRequest>,
    cancellation: &CancellationToken,
) -> Result<domain::MessageStatus, Error> {
    let tools = dependencies
        .read_tools
        .call()
        .await
        .map_err(Error::caused_by)?;
    let workspace = conversation.workspace.clone().ok_or_else(|| {
        Error::caused_by("Resume this conversation with an explicit workspace before using tools")
    })?;
    let deadline = std::time::Instant::now();
    let mut executions = 0;
    for step_number in 0..domain::MAX_MODEL_STEPS {
        while let Ok(request) = steering.try_recv() {
            accept(dependencies, conversation, request, buffer, cancellation).await?;
        }
        if cancellation.is_cancelled() {
            return Ok(domain::MessageStatus::Cancelled);
        }
        if deadline.elapsed().as_secs() > 1200 {
            return Err(Error::caused_by("Turn exceeded 20 minutes"));
        }
        conversation.model = model_access::read_model(
            &dependencies.read_models,
            &conversation.model.provider_id,
            &conversation.model.id,
            conversation.effort,
        )
        .await
        .map_err(Error::caused_by)?;
        model_access::read(
            &dependencies.read_model_allowlist,
            &dependencies.read_provider_connection,
            &dependencies.create_model_step_factory,
            &conversation.model,
        )
        .await
        .map_err(Error::caused_by)?;
        let reply = conversation.active_reply_mut().map_err(Error::caused_by)?;
        reply.content = buffer.lock().clone();
        reply.begin_generation().map_err(Error::caused_by)?;
        checkpoint(dependencies, conversation).await?;
        let finalizing =
            step_number + 1 == domain::MAX_MODEL_STEPS || executions >= domain::MAX_TOOL_EXECUTIONS;
        let step_token = cancellation.child_token();
        let future = create.call(
            conversation.clone(),
            if finalizing {
                Vec::new()
            } else {
                tools.clone()
            },
            finalizing,
            present.clone(),
            step_token.clone(),
        );
        tokio::pin!(future);
        let mut restart = false;
        let result = loop {
            tokio::select! {
                result=&mut future=>break result,
                Some(request)=steering.recv()=>{
                    if finalizing {
                        let accepted = conversation.steering_accepted(&request.id, &request.text).unwrap_or(false);
                        let _ = request.accepted.send(accepted);
                        continue;
                    }
                    if accept(dependencies,conversation,request,buffer,cancellation).await?{
                        step_token.cancel();restart=true;break (&mut future).await;
                    }
                }
            }
        };
        let reply = conversation.active_reply_mut().map_err(Error::caused_by)?;
        if result
            .as_ref()
            .is_err_and(|error| error.not_attempted && error.reported_usage.is_none())
        {
            reply
                .discard_generation_before_attempt()
                .map_err(Error::caused_by)?;
        } else {
            reply
                .report_generation(match &result {
                    Ok(step) => step.usage,
                    Err(error) => error.reported_usage,
                })
                .map_err(Error::caused_by)?;
        }
        if restart || cancellation.is_cancelled() || result.is_err() {
            checkpoint(dependencies, conversation).await?;
            present
                .call(domain::TurnEvent::Usage {
                    usage: conversation.usage(),
                })
                .map_err(Error::caused_by)?;
            if restart {
                continue;
            }
            return result
                .map(|_| domain::MessageStatus::Cancelled)
                .map_err(Error::caused_by);
        }
        let step = result.map_err(Error::caused_by)?;
        let status = step.status;
        let calls = conversation
            .record_step(step, buffer.lock().clone())
            .map_err(Error::caused_by)?;
        checkpoint(dependencies, conversation).await?;
        present
            .call(domain::TurnEvent::Usage {
                usage: conversation.usage(),
            })
            .map_err(Error::caused_by)?;
        if calls.is_empty() {
            return Ok(if finalizing {
                domain::MessageStatus::Limited
            } else {
                status
            });
        }
        let message_id = conversation
            .active_reply()
            .map_err(Error::caused_by)?
            .id
            .clone();
        for call in &calls {
            present
                .call(domain::TurnEvent::Tool {
                    message_id: message_id.clone(),
                    call: call.clone(),
                })
                .map_err(Error::caused_by)?;
        }
        let mut steered = false;
        for mut call in calls {
            while let Ok(request) = steering.try_recv() {
                steered |=
                    accept(dependencies, conversation, request, buffer, cancellation).await?;
            }
            let invocation = domain::Tool::parse(&call.name)
                .filter(|tool| tools.contains(tool))
                .map(|tool| {
                    dependencies
                        .read_tool_invocation
                        .call(tool, call.arguments.clone())
                });
            let result;
            if cancellation.is_cancelled()
                || steered
                || finalizing
                || executions >= domain::MAX_TOOL_EXECUTIONS
            {
                result = domain::ToolResult {
                    status: domain::ToolStatus::Skipped,
                    output: "Not executed: cancellation, steering, or execution budget reached."
                        .into(),
                    artifact: None,
                    truncated: false,
                };
            } else if let Some(Ok(invocation)) = invocation {
                executions += 1;
                call.start().map_err(Error::caused_by)?;
                conversation
                    .update_tool_call(call.clone())
                    .map_err(Error::caused_by)?;
                checkpoint(dependencies, conversation).await?;
                present
                    .call(domain::TurnEvent::Tool {
                        message_id: message_id.clone(),
                        call: call.clone(),
                    })
                    .map_err(Error::caused_by)?;
                let started = std::time::Instant::now();
                let future = execute(
                    dependencies,
                    workspace.clone(),
                    invocation,
                    cancellation.clone(),
                );
                tokio::pin!(future);
                result = loop {
                    tokio::select! {
                        result=&mut future=>break result,
                        Some(request)=steering.recv()=>{steered|=accept(dependencies,conversation,request,buffer,cancellation).await?;}
                    }
                };
                call.elapsed_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            } else {
                result = domain::ToolResult {
                    status: domain::ToolStatus::Invalid,
                    output: match invocation {
                        Some(Err(error)) => error_context::diagnostic(&error),
                        _ => "Unknown or unavailable tool".into(),
                    },
                    artifact: None,
                    truncated: false,
                };
            }
            call.finish(result).map_err(Error::caused_by)?;
            let call = dependencies
                .create_tool_output
                .call(conversation.id.clone(), call)
                .await
                .map_err(Error::caused_by)?;
            conversation
                .update_tool_call(call.clone())
                .map_err(Error::caused_by)?;
            checkpoint(dependencies, conversation).await?;
            present
                .call(domain::TurnEvent::Tool {
                    message_id: message_id.clone(),
                    call: call.clone(),
                })
                .map_err(Error::caused_by)?;
            if call.status == domain::ToolStatus::Unknown {
                return Err(Error::caused_by(
                    "Tool cleanup or outcome is uncertain; stopped the turn",
                ));
            }
        }
        if finalizing {
            return Ok(domain::MessageStatus::Limited);
        }
    }
    Ok(domain::MessageStatus::Limited)
}
async fn execute(
    dependencies: &Dependencies,
    workspace: std::path::PathBuf,
    invocation: domain::ToolInvocation,
    cancellation: CancellationToken,
) -> domain::ToolResult {
    let result: Result<domain::ToolResult, Box<dyn std::error::Error + Send + Sync>> = async {
        Ok(match invocation {
            domain::ToolInvocation::ReadFile(arguments) => {
                dependencies
                    .read_file
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::CreateProcess(arguments) => {
                dependencies
                    .create_process
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::UpdateFile(arguments) => {
                dependencies
                    .update_file
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::CreateFile(arguments) => {
                dependencies
                    .create_file
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::ReadDirectory(arguments) => {
                dependencies
                    .read_directory
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::ReadPaths(arguments) => {
                dependencies
                    .read_paths
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::ReadTextMatches(arguments) => {
                dependencies
                    .read_text_matches
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::ReadCodeMatches(arguments) => {
                dependencies
                    .read_code_matches
                    .call(workspace, arguments, cancellation)
                    .await?
            }
            domain::ToolInvocation::ReadWebSearch(arguments) => {
                dependencies
                    .read_web_search
                    .call(workspace, arguments, cancellation)
                    .await?
            }
        })
    }
    .await;
    result.unwrap_or_else(|error| {
        domain::ToolResult::failed(error_context::diagnostic(error.as_ref()))
    })
}
