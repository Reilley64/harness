use crate::ports::PortFuture;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to discover accepted execution claims")]
pub struct ReadExecutionClaimsError;

port! { pub ReadExecutionClaims(parent_id:String,cancellation:CancellationToken,)->PortFuture<Result<Vec<domain::ChildExecution>,ReadExecutionClaimsError>>; }
