use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read text matches")]
pub struct ReadTextMatchesError;
port! { pub ReadTextMatches(workspace: std::path::PathBuf, arguments: domain::ReadTextMatchesArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, ReadTextMatchesError>>; }
