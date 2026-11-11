use crate::ports::PortFuture;
use tokio_util::sync::CancellationToken;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read work proposal")]
pub struct ReadWorkProposalError;
port! { pub ReadWorkProposal(workspace: std::path::PathBuf, path: String, cancellation: CancellationToken) -> PortFuture<Result<domain::WorkProposal, ReadWorkProposalError>>; }
