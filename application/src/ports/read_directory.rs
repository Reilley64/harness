use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read directory")]
pub struct ReadDirectoryError;
port! { pub ReadDirectory(workspace: std::path::PathBuf, arguments: domain::ReadDirectoryArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, ReadDirectoryError>>; }
