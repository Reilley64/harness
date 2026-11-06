use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read web search")]
pub struct ReadWebSearchError;
port! { pub ReadWebSearch(workspace: std::path::PathBuf, arguments: domain::ReadWebSearchArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, ReadWebSearchError>>; }
