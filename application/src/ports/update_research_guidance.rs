use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update research guidance")]
pub struct UpdateResearchGuidanceError;

port! { pub UpdateResearchGuidance(guidance: domain::ResearchGuidance, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ResearchGuidance, UpdateResearchGuidanceError>>; }
