mod record;

use self::record::Record;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn read_execution_claim(
    directory: &Path,
) -> impl Fn(String, CancellationToken) -> PortFuture<Result<Option<domain::ChildExecution>, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |intent, cancellation| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&intent)?;
            match tokio::fs::symlink_metadata(&directory).await {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
            }
            storage::private_directory(&directory).await?;
            let lock_path = directory.join(format!("{intent}.lock"));
            let guard = loop {
                if cancellation.is_cancelled() {
                    return Err("Execution claim inspection stopped".into());
                }
                match storage::lock(&lock_path).await {
                    Ok(guard) => break guard,
                    Err(error)
                        if error
                            .source()
                            .and_then(|source| source.downcast_ref::<std::fs::TryLockError>())
                            .is_some_and(|cause| {
                                matches!(cause, std::fs::TryLockError::WouldBlock)
                            }) =>
                    {
                        tokio::select! {
                            _ = cancellation.cancelled() => return Err("Execution claim inspection stopped".into()),
                            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
                        }
                    }
                    Err(error) => return Err(error),
                }
            };
            let path = directory.join(format!("{intent}.json"));
            match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
            }
            let bytes = storage::read_private(&path, 4 * 1024 * 1024).await?;
            tokio::task::spawn_blocking(move || {
                let _guard = guard;
                let child: domain::ChildExecution =
                    serde_json::from_slice::<Record>(&bytes)?.try_into()?;
                if child.intent != intent {
                    return Err("Execution claim differs from its key".into());
                }
                Ok(Some(child))
            })
            .await?
        })
    }
}
