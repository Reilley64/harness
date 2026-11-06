#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("invalid tool arguments")]
pub struct ReadToolInvocationError;
port! { pub ReadToolInvocation(tool: domain::Tool, arguments: String,) -> Result<domain::ToolInvocation, ReadToolInvocationError>; }
