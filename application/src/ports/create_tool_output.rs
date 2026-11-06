use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to persist tool output")]
pub struct CreateToolOutputError;
port! { pub CreateToolOutput(conversation_id: String, call: domain::ToolCall,) -> PortFuture<Result<domain::ToolCall, CreateToolOutputError>>; }
