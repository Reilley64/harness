use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read code matches")]
pub struct ReadCodeMatchesError;
port! { pub ReadCodeMatches(workspace: std::path::PathBuf, arguments: domain::ReadCodeMatchesArguments, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::ToolResult, ReadCodeMatchesError>>; }
