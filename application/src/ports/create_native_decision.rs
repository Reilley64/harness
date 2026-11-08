use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create native decision")]
pub struct CreateNativeDecisionError {
    pub reported_response: Option<domain::DecisionResponse>,
}

impl CreateNativeDecisionError {
    pub fn with_report(mut self, report: domain::DecisionResponse) -> Self {
        self.reported_response = Some(report);
        self
    }
}

port! { pub CreateNativeDecision(decision: domain::AssignmentDecision, cancellation: tokio_util::sync::CancellationToken) -> PortFuture<Result<domain::DecisionResponse, CreateNativeDecisionError>>; }
