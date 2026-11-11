use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read decision")]
pub struct ReadDecisionError;

port! { pub ReadDecision(id: String) -> PortFuture<Result<Option<domain::AssignmentDecision>, ReadDecisionError>>; }
