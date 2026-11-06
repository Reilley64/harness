mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    selection: domain::ModelSelection,
    workspace: std::path::PathBuf,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let model = dependencies
        .read_models
        .call()
        .await
        .map_err(Error::caused_by)?
        .into_iter()
        .find(|model| model.id == selection.model_id && model.provider_id == selection.provider_id)
        .ok_or_else(|| {
            Error::caused_by("The selected model is unavailable; choose a model with /model")
        })?;
    model
        .validate_effort(selection.effort)
        .map_err(Error::caused_by)?;
    let connection = dependencies
        .read_provider_connection
        .call(selection.provider_id)
        .await
        .map_err(Error::caused_by)?;
    let conversation = dependencies
        .create_conversation
        .call(model, selection.effort, connection, workspace)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { conversation })
}
