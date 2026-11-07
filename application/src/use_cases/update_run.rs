use crate::ports::PresentTurnEvent;
use crate::turn_control::SteeringRequest;
use crate::use_cases::create_message;
use tokio_util::sync::CancellationToken;

pub use create_message::Dependencies;
pub use create_message::Error;
pub use create_message::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    present: PresentTurnEvent,
    steering: tokio::sync::mpsc::Receiver<SteeringRequest>,
    workspace: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    create_message::resume(
        dependencies.into(),
        id,
        present,
        steering,
        workspace,
        cancellation,
    )
    .await
}
