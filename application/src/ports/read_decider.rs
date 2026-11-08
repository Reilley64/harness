use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read decider")]
pub struct ReadDeciderError;

port! { pub ReadDecider() -> PortFuture<Result<Option<String>, ReadDeciderError>>; }
