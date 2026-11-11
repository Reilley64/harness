use crate::ports::PortFuture;
use crate::ports::PresentTurnEvent;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create model step")]
pub struct CreateModelStepError {
    pub reported_usage: Option<domain::TokenUsage>,
    pub reported_cost: Option<f64>,
    pub not_attempted: bool,
    pub provider_failure: Option<Box<domain::ProviderFailure>>,
}

impl CreateModelStepError {
    pub fn with_provider_failure(mut self, failure: Option<domain::ProviderFailure>) -> Self {
        self.provider_failure = failure.map(Box::new);
        self
    }

    pub fn without_attempt(mut self) -> Self {
        self.not_attempted = self.reported_usage.is_none() && self.reported_cost.is_none();
        self
    }

    pub fn with_usage(mut self, usage: Option<domain::TokenUsage>) -> Self {
        self.reported_usage = usage;
        if usage.is_some() {
            self.not_attempted = false;
        }
        self
    }
}
port! { pub CreateModelStep(conversation: domain::Conversation, tools: Vec<domain::Tool>, finalizing: bool, present: PresentTurnEvent, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ModelStep, CreateModelStepError>>; }
