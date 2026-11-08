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
                    let (child, encoded) =
                        tokio::task::spawn_blocking(move || record::claim(bytes, *child)).await??;
                    let fresh = encoded.is_some();
                    if let Some(encoded) = encoded {
                        if cancellation.is_cancelled() {
                            return Err(
                                "Dispatch stopped before durable execution claim admission".into(),
                            );
                        }
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
                    None => domain::Coordination::new(id.clone()),
                };
                if coordination.agent_id != id {
                    return Err::<_, Error>("Coordination identity differs from its file".into());
                }
                let expected = (coordination.revision != 0).then_some(coordination.revision);
                let independent = matches!(&command, domain::CoordinationCommand::Dispatch(_) | domain::CoordinationCommand::Start { .. } | domain::CoordinationCommand::Receive { .. } | domain::CoordinationCommand::CompleteFollowUp { .. } | domain::CoordinationCommand::Release { .. });
                if matches!(&command, domain::CoordinationCommand::Deliver { .. }) && revision != expected {
                    return Err(RevisionConflict::caused_by("Current-work delivery revision changed before admission").into());
                }
                let dispatch = match &command {
                    domain::CoordinationCommand::Dispatch(child) => Some((child.execution_id.clone(), child.controller.clone(), child.agent_id.clone())),
                    _ => None,
                };
                let changed = coordination.apply(command)?;
                let claimed = changed && fresh_intent && dispatch.is_some();
                if !changed {
                    return Ok((CoordinationResponse { coordination, changed, claimed }, None));
                }
                if let Some((execution_id, controller, conversation_id)) = dispatch
                    && !fresh_intent
                {
                    coordination.apply(domain::CoordinationCommand::Receive {
                        execution_id: execution_id.clone(),
                        controller,
                        outcome: domain::ChildOutcome {
                            id: execution_id,
                            status: domain::ChildExecutionStatus::Unknown,
                            conversation_id,
                            run_id: None,
                            reply_id: None,
                            summary: "A durable execution claim exists without its parent admission record. No child was replayed. Inspect and explicitly reconcile; process death, a missing record or a heartbeat is not proof of unsent effects.".into(),
                            artifacts: Vec::new(),
                        },
                    })?;
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
            }
            Ok(response)
        })
    }
}
