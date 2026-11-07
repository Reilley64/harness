mod dependencies;
mod error;
mod output;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    workspace: std::path::PathBuf,
    id: Option<String>,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let conversation = if let Some(id) = id {
        Some(
            dependencies
                .read_conversation
                .call(id)
                .await
                .map_err(Error::caused_by)?,
        )
    } else {
        None
    };
    let workspace = conversation
        .as_ref()
        .and_then(|c| c.workspace.clone())
        .unwrap_or(workspace);
    let status = dependencies
        .read_setup
        .call(workspace, cancellation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output {
        status,
        conversation,
    })
}
