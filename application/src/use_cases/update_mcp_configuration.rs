use crate::ports::UpdateMcpConfiguration;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) update_mcp_configuration: UpdateMcpConfiguration,
}

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("MCP configuration request failed")]
pub struct Error;

pub struct Output {
    pub configuration: domain::McpConfiguration,
}

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
