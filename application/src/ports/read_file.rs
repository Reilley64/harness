use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read file")]
pub struct ReadFileError;
port! { pub ReadFile(workspace: std::path::PathBuf, arguments: domain::ReadFileArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, ReadFileError>>; }
