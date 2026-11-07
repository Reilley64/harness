mod dependencies;
mod error;
mod output;

use crate::error_context::ResultExt;
use crate::model_access;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    selection: domain::ModelSelection,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let model = dependencies
        .read_models
        .call()
        .await
        .map_err(Error::caused_by)?
        .into_iter()
        .find(|model| model.id == selection.model_id && model.provider_id == selection.provider_id)
        .ok_or("The selected model is missing from the catalog")
        .context("Capability blocked: the selected model is unavailable; choose a supported model with /model")
        .map_err(Error::caused_by)?;
    model_access::read(
        &dependencies.read_model_allowlist,
        &dependencies.read_provider_connection,
        &dependencies.create_model_step_factory,
        &model,
    )
    .await
    .map_err(Error::caused_by)?;
    let selected_model = domain::SelectedModel::new(model, selection.effort)
        .context("Capability blocked: this effort is unsupported; choose an effort with /model")
        .map_err(Error::caused_by)?;
    let selected_model = dependencies
        .update_model_selection
        .call(selected_model)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { selected_model })
}
