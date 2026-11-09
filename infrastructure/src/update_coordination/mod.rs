mod arguments;
mod record;
pub(crate) use arguments::decode;
pub(crate) use arguments::definition;

use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::storage;
use application::ports::CoordinationResponse;
use application::ports::PortFuture;
use std::path::Path;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[ohno::error]
#[display("Coordination revision changed")]
pub(crate) struct RevisionConflict;

async fn lock(path: &Path, cancellation: &CancellationToken) -> Result<std::fs::File, Error> {
    loop {
        if cancellation.is_cancelled() {
            return Err("Execution claim stopped before lock admission".into());
        }
        match storage::lock(path).await {
            Ok(guard) => return Ok(guard),
            Err(error)
                if error
                    .source()
                    .and_then(|cause| cause.downcast_ref::<std::fs::TryLockError>())
                    .is_some_and(|cause| matches!(cause, std::fs::TryLockError::WouldBlock)) =>
            {
                tokio::select! {
                    _ = cancellation.cancelled() => return Err("Execution claim stopped while waiting for its lock".into()),
                    _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
                }
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn update_coordination(
    directory: &Path,
) -> impl Fn(
    String,
    domain::CoordinationCommand,
    Option<u64>,
    CancellationToken,
) -> PortFuture<Result<CoordinationResponse, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |id, command, revision, cancellation| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&id)?;
            let (intent_guard, command, fresh_intent) = match command {
                domain::CoordinationCommand::Dispatch(child) => {
                    conversations::validate_id(&child.intent)?;
                    let intent_path = directory
                        .join("intents")
                        .join(format!("{}.json", child.intent));
                    let guard = lock(&intent_path, &cancellation).await?;
                    let bytes = match tokio::fs::symlink_metadata(&intent_path).await {
                        Ok(_) => Some(storage::read_private(&intent_path, 4 * 1024 * 1024).await?),
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                        Err(error) => return Err(error.into()),
                    };
                    let guidance_path = directory
                        .join("guidance")
                        .join(format!("{}.json", child.intent));
                    let adoption = match tokio::fs::symlink_metadata(&guidance_path).await {
                        Ok(_) => {
                            Some(storage::read_private(&guidance_path, 2 * 1024 * 1024).await?)
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                        Err(error) => return Err(error.into()),
                    };
                    let (child, encoded) =
                        tokio::task::spawn_blocking(move || record::claim(bytes, *child, adoption))
                            .await??;
                    let fresh = encoded.is_some();
                    if let Some(encoded) = encoded {
                        if cancellation.is_cancelled() {
                            return Err(
                                "Dispatch stopped before durable execution claim admission".into(),
                            );
                        }
                        let index = directory.join("parents").join(&child.parent_id);
                        storage::private_directory(&index).await?;
                        let reference = record::reference(&child)?;
                        storage::replace(&index.join(format!("{}.json", child.intent)), &reference)
                            .await?;
                        storage::replace(&intent_path, &encoded).await?;
                    }
                    (
                        Some(guard),
                        domain::CoordinationCommand::Dispatch(Box::new(child)),
                        fresh,
                    )
                }
                command => (None, command, false),
            };
            let _intent_guard = intent_guard;
            let path = directory.join(format!("{id}.json"));
            let _guard = loop {
                if cancellation.is_cancelled() {
                    return Err("Coordination stopped before storage admission".into());
                }
                match storage::lock(&path).await {
                    Ok(guard) => break guard,
                    Err(error)
                        if error
                            .source()
                            .and_then(|cause| cause.downcast_ref::<std::fs::TryLockError>())
                            .is_some_and(|cause| {
                                matches!(cause, std::fs::TryLockError::WouldBlock)
                            }) =>
                    {
                        tokio::select! {
                            _ = cancellation.cancelled() => return Err("Coordination stopped while waiting for storage".into()),
                            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
                        }
                    }
                    Err(error) => return Err(error),
                }
            };
            let bytes = match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => Some(storage::read_private(&path, 4 * 1024 * 1024).await?),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(error.into()),
            };
            let (response, encoded) = tokio::task::spawn_blocking(move || {
                let mut coordination: domain::Coordination = match bytes {
                    Some(bytes) => serde_json::from_slice::<Record>(&bytes)?.try_into()?,
                    None => {
                        let agent_id = match &command {
                            domain::CoordinationCommand::Acquire { agent_id, .. } => agent_id.clone(),
                            domain::CoordinationCommand::RetainClaim(child) => child.parent_agent_id.clone(),
                            _ => return Err::<_, Error>("Parent coordination must first be admitted by its actual active execution".into()),
                        };
                        let mut coordination = domain::Coordination::new(id.clone(), agent_id);
                        if matches!(&command, domain::CoordinationCommand::RetainClaim(_)) { coordination.gate = domain::CoordinationGate::Unknown; coordination.authority_complete = false; }
                        coordination
                    }
                };
                if coordination.conversation_id != id {
                    return Err::<_, Error>("Coordination identity differs from its file".into());
                }
                let expected = (coordination.revision != 0).then_some(coordination.revision);
                let independent = matches!(&command, domain::CoordinationCommand::AnswerChild { .. } | domain::CoordinationCommand::AdmitChildAction { .. } | domain::CoordinationCommand::SteerChild { .. } | domain::CoordinationCommand::ApplyChildInstruction { .. } | domain::CoordinationCommand::Dispatch(_) | domain::CoordinationCommand::Start { .. } | domain::CoordinationCommand::Receive { .. } | domain::CoordinationCommand::CompleteFollowUp { .. } | domain::CoordinationCommand::Release { .. } | domain::CoordinationCommand::Wait { .. } | domain::CoordinationCommand::Gate { .. } | domain::CoordinationCommand::RetainClaim(_) | domain::CoordinationCommand::StopChild { .. } | domain::CoordinationCommand::Stop { .. } | domain::CoordinationCommand::MarkUnknown { .. });
                if matches!(&command, domain::CoordinationCommand::Deliver { .. }) && revision != expected {
                    return Err(RevisionConflict::caused_by("Current-work delivery revision changed before admission").into());
                }
                let command = match command {
                    domain::CoordinationCommand::Dispatch(mut child)
                        if !fresh_intent && !coordination.children.iter().any(|prior| prior.intent == child.intent) => {
                        child.status = domain::ChildExecutionStatus::Unknown;
                        child.outcome = Some(domain::ChildOutcome {
                            id: child.execution_id.clone(),
                            status: domain::ChildExecutionStatus::Unknown,
                            conversation_id: child.agent_id.clone(),
                            run_id: None,
                            reply_id: None,
                            summary: "A durable execution claim exists without its parent admission record. No child was replayed. Inspect and explicitly reconcile; process death, a missing record or a heartbeat is not proof of unsent effects.".into(),
                            artifacts: Vec::new(),
                        });
                        domain::CoordinationCommand::RetainClaim(child)
                    }
                    command => command,
                };
                let dispatch = matches!(&command, domain::CoordinationCommand::Dispatch(_));
                let changed = coordination.apply(command)?;
                let claimed = changed && fresh_intent && dispatch;
                if !changed {
                    return Ok((CoordinationResponse { coordination, changed, claimed }, None));
                }
                if revision != expected && !independent {
                    return Err(RevisionConflict::caused_by("Coordination changed; inspect its current revision before another transition").into());
                }
                coordination.revision = coordination.revision.checked_add(1)
                    .ok_or("Coordination revision exhausted")?;
                let encoded = encode_bounded(&Record::from(coordination.clone()), 4 * 1024 * 1024)?;
                let _: domain::Coordination = serde_json::from_slice::<Record>(&encoded)?.try_into()?;
                Ok((CoordinationResponse { coordination, changed, claimed }, Some(encoded)))
            }).await??;
            if let Some(encoded) = encoded {
                if cancellation.is_cancelled() {
                    return Err("Coordination stopped before atomic write admission".into());
                }
                storage::replace(&path, &encoded).await?;
                let origin = path.with_extension("origin");
                if tokio::fs::symlink_metadata(&origin)
                    .await
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
                {
                    storage::replace(&origin, b"coordination-format-two-initialized").await?;
                }
            }
            Ok(response)
        })
    }
}
