use crate::ports::PortFuture;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to prepare Setup draft")]
pub struct ReadSetupDraftError;

port! {
    pub ReadSetupDraft(workspace: std::path::PathBuf, arguments: domain::CreateFileArguments, proposed_wiki: Option<String>, cancellation: CancellationToken,) -> PortFuture<Result<domain::SetupDraft, ReadSetupDraftError>>;
}
