mod dependencies;
mod error;
mod output;

use crate::error_context::ResultExt;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(dependencies: impl Into<Dependencies>) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let policy = dependencies
        .read_model_allowlist
        .call()
        .await
        .context("The saved model allowlist is invalid or unavailable; no inference is permitted")
        .map_err(Error::caused_by)?;
    Ok(Output { policy })
}
