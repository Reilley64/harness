use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update decider")]
pub struct UpdateDeciderError;

port! { pub UpdateDecider(model_id: String) -> PortFuture<Result<String, UpdateDeciderError>>; }
