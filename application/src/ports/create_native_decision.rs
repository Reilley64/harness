use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create native decision")]
pub struct CreateNativeDecisionError;

port! { pub CreateNativeDecision(decision: domain::AssignmentDecision, cancellation: tokio_util::sync::CancellationToken) -> PortFuture<Result<domain::DecisionResponse, CreateNativeDecisionError>>; }
