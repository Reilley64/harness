use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update decision")]
pub struct UpdateDecisionError;

port! { pub UpdateDecision(decision: domain::AssignmentDecision, revision: Option<u64>) -> PortFuture<Result<domain::AssignmentDecision, UpdateDecisionError>>; }
