use crate::ports::PortFuture;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("MCP configuration operation failed")]
pub struct UpdateMcpConfigurationError;
port! { pub UpdateMcpConfiguration(workspace: PathBuf, action: domain::McpConfigurationAction, cancellation: CancellationToken,) -> PortFuture<Result<domain::McpConfiguration, UpdateMcpConfigurationError>>; }
