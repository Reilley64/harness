use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read work proposal")]
pub struct ReadWorkProposalError;
port! { pub ReadWorkProposal(workspace: std::path::PathBuf, path: String) -> PortFuture<Result<domain::WorkProposal, ReadWorkProposalError>>; }
