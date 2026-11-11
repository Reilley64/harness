use crate::ports::PortFuture;
use crate::ports::PresentMcpAuthorization;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("MCP configuration operation failed")]
pub struct UpdateMcpConfigurationError;
#[derive(Debug)]
pub struct Response {
    pub configuration: domain::McpConfiguration,
    pub diagnostics: Vec<Arc<dyn std::error::Error + Send + Sync>>,
}

port! { pub UpdateMcpConfiguration(workspace: PathBuf, action: domain::McpConfigurationAction, present_authorization: Option<PresentMcpAuthorization>, cancellation: CancellationToken,) -> PortFuture<Result<Response, UpdateMcpConfigurationError>>; }
