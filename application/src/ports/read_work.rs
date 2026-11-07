use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read work")]
pub struct ReadWorkError;
port! { pub ReadWork(id: String) -> PortFuture<Result<Option<domain::ApprovedWork>, ReadWorkError>>; }
