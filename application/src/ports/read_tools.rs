use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read available tools")]
pub struct ReadToolsError;
port! { pub ReadTools() -> PortFuture<Result<Vec<domain::Tool>, ReadToolsError>>; }
