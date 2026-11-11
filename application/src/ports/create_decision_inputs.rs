use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create decision inputs")]
pub struct CreateDecisionInputsError;

port! { pub CreateDecisionInputs(decision: domain::AssignmentDecision, members: Vec<String>, level: u32) -> PortFuture<Result<Vec<domain::DecisionStage>, CreateDecisionInputsError>>; }
