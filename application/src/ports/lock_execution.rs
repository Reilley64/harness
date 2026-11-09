use crate::ports::PortFuture;

pub struct ExecutionGuard(pub Box<dyn Send + Sync>);

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to claim the execution owner")]
pub struct LockExecutionError {
    pub busy: bool,
}

port! {
    pub LockExecution(id: String,) -> PortFuture<Result<ExecutionGuard, LockExecutionError>>;
}
