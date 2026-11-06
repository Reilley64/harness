mod dependencies;
mod error;
mod output;

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
    Ok(connection.into())
}
