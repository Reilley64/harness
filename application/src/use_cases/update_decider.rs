mod dependencies;
mod error;
mod output;

use crate::decision_access;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    model_id: String,
) -> Result<Output, Error> {
    let d = dependencies.into();
    decision_access::native(
        &d.read_models,
        &d.read_model_allowlist,
        &d.read_provider_connection,
        &model_id,
    )
    .await
    .map_err(Error::caused_by)?;

    let selected = d
        .update_decider
        .call(model_id)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { selected })
}
