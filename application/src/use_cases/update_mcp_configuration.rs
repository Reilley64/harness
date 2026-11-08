mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    workspace: PathBuf,
    action: domain::McpConfigurationAction,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let configuration = dependencies
        .into()
        .update_mcp_configuration
        .call(workspace, action, cancellation)
        .await
        .map_err(Error::caused_by)?;

    Ok(Output { configuration })
}
