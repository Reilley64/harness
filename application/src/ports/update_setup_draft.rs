use crate::ports::PortFuture;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to apply Setup draft")]
pub struct UpdateSetupDraftError;

port! {
    pub UpdateSetupDraft(draft: domain::SetupDraft, cancellation: CancellationToken,) -> PortFuture<Result<domain::ToolResult, UpdateSetupDraftError>>;
}
