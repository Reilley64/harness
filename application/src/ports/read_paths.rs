use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read paths")]
pub struct ReadPathsError;
port! { pub ReadPaths(workspace: std::path::PathBuf, arguments: domain::ReadPathsArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, ReadPathsError>>; }
