use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read coordination")]
pub struct ReadCoordinationError;

port! { pub ReadCoordination(conversation_id: String,) -> PortFuture<Result<Option<domain::Coordination>, ReadCoordinationError>>; }
