use crate::ports::PortFuture;
use tokio_util::sync::CancellationToken;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to inspect repository Setup outputs")]
pub struct ReadSetupError;

port! {
    pub ReadSetup(workspace: std::path::PathBuf, cancellation: CancellationToken,) -> PortFuture<Result<domain::SetupStatus, ReadSetupError>>;
}
