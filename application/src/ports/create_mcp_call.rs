use crate::ports::PortFuture;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("MCP execution failed")]
pub struct CreateMcpCallError;
port! { pub CreateMcpCall(workspace: PathBuf, arguments: domain::McpCallArguments, cancellation: CancellationToken,) -> PortFuture<Result<domain::ToolResult, CreateMcpCallError>>; }
