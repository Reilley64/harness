use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create file")]
pub struct CreateFileError;
port! { pub CreateFile(workspace: std::path::PathBuf, arguments: domain::CreateFileArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, CreateFileError>>; }
