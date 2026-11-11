use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read routing evidence")]
pub struct ReadRoutingEvidenceError;

pub struct Response {
    pub evidence: Vec<(String, Vec<domain::RoutingEvidence>)>,
    pub diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

port! { pub ReadRoutingEvidence(candidates: Vec<domain::DecisionCandidate>, now: u64, refresh: bool, cancellation: tokio_util::sync::CancellationToken) -> PortFuture<Result<Response, ReadRoutingEvidenceError>>; }
