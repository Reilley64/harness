use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read research guidance")]
pub struct ReadResearchGuidanceError;

port! { pub ReadResearchGuidance(intent: String,) -> PortFuture<Result<Option<domain::ResearchGuidance>, ReadResearchGuidanceError>>; }
