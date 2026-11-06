use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update file")]
pub struct UpdateFileError;
port! { pub UpdateFile(workspace: std::path::PathBuf, arguments: domain::UpdateFileArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, UpdateFileError>>; }
