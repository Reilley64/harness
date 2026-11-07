mod dependencies;
mod error;
mod output;

use crate::error_context::ResultExt;
use crate::ports::ModelAllowlistUpdate;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::ports::PresentAuthorization;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    provider_id: domain::ProviderId,
    present_authorization: PresentAuthorization,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let create_provider_connection = dependencies
        .create_provider_connection_factory
        .call(provider_id)
        .map_err(Error::caused_by)?;
    let connection = create_provider_connection
        .call(present_authorization)
        .await
        .map_err(Error::caused_by)?;
    dependencies
        .update_model_allowlist
        .call(ModelAllowlistUpdate::InitializeProvider(
            connection.provider_id.clone(),
        ))
        .await
        .context("Provider credentials were saved, but model policy initialization failed; reload /model settings before inference")
        .map_err(Error::caused_by)?;
    Ok(connection.into())
}
