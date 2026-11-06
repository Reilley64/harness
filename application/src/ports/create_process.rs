use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create process")]
pub struct CreateProcessError;
port! { pub CreateProcess(workspace: std::path::PathBuf, arguments: domain::CreateProcessArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, CreateProcessError>>; }
