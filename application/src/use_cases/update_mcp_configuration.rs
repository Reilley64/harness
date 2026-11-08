mod dependencies;
mod error;
mod output;

use crate::ports::PresentMcpAuthorization;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    workspace: PathBuf,
    action: domain::McpConfigurationAction,
    present_authorization: Option<PresentMcpAuthorization>,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let response = dependencies
        .into()
        .update_mcp_configuration
        .call(workspace, action, present_authorization, cancellation)
        .await
        .map_err(Error::caused_by)?;

    Ok(Output {
        configuration: response.configuration,
        diagnostics: response.diagnostics,
    })
}
