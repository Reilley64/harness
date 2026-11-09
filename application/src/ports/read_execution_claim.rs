use crate::ports::PortFuture;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read execution claim")]
pub struct ReadExecutionClaimError;

port! { pub ReadExecutionClaim(intent: String, cancellation: CancellationToken,) -> PortFuture<Result<Option<domain::ChildExecution>, ReadExecutionClaimError>>; }
