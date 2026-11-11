mod dependencies;
mod error;
mod output;

use crate::error_context::ResultExt;
use crate::ports::ModelAllowlistUpdate;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    patterns: Vec<String>,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let policy = domain::ModelAllowlist::new(patterns).context("Use at most 256 positive provider/model patterns, at most 512 bytes each; only * is a wildcard and ! is not supported").map_err(Error::caused_by)?;
    let policy = dependencies
        .update_model_allowlist
        .call(ModelAllowlistUpdate::Replace(policy))
        .await
        .context("Could not save the model allowlist; reload /model settings before retrying")
        .map_err(Error::caused_by)?;
    Ok(Output { policy })
}
