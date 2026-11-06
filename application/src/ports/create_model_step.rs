use crate::ports::PortFuture;
use crate::ports::PresentTurnEvent;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create model step")]
pub struct CreateModelStepError {
    pub reported_usage: Option<domain::TokenUsage>,
}

impl CreateModelStepError {
    pub fn with_usage(mut self, usage: Option<domain::TokenUsage>) -> Self {
        self.reported_usage = usage;
        self
    }
}
port! { pub CreateModelStep(conversation: domain::Conversation, tools: Vec<domain::Tool>, finalizing: bool, present: PresentTurnEvent, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ModelStep, CreateModelStepError>>; }
