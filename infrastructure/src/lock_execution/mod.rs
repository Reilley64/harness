use crate::conversations;
use crate::storage;
use application::ports::ExecutionGuard;
use application::ports::LockExecutionError;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) fn error(original: Error) -> LockExecutionError {
    let busy = original
        .source()
        .and_then(|cause| cause.downcast_ref::<std::fs::TryLockError>())
        .is_some_and(|cause| matches!(cause, std::fs::TryLockError::WouldBlock));
    let mut error = LockExecutionError::from(original);
    error.busy = busy;
    error
}

pub fn lock_execution(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<ExecutionGuard, Error>> + Send + Sync + use<> {
    let directory = directory.to_owned();
    move |id| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&id)?;
            let guard = storage::lock(&directory.join(format!("{id}.json"))).await?;
            Ok(ExecutionGuard(Box::new(guard)))
        })
    }
}
