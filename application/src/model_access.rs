use crate::error_context::ResultExt;
use crate::ports::CreateModelStepFactory;
use crate::ports::ReadModelAllowlist;
use crate::ports::ReadModels;
use crate::ports::ReadProviderConnection;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("Model configuration required")]
pub(crate) struct Error;

pub(crate) async fn read_model(
    catalog: &ReadModels,
    provider: &domain::ProviderId,
    id: &str,
    effort: Option<domain::ModelEffort>,
) -> Result<domain::Model, Error> {
    let model = catalog.call().await
        .context("Capability blocked: the model catalog could not be read; retry or choose a supported model with /model")
        .map_err(Error::caused_by)?
        .into_iter()
        .find(|model| model.provider_id == *provider && model.id == id)
        .ok_or("The selected model is missing from the catalog")
        .context("Capability blocked: the selected model is unavailable; choose a supported model with /model")
        .map_err(Error::caused_by)?;
    model
        .validate_effort(effort)
        .context("Capability blocked: this effort is unsupported; choose an effort with /model")
        .map_err(Error::caused_by)?;
    Ok(model)
}

pub(crate) async fn read(
    policy: &ReadModelAllowlist,
    connection: &ReadProviderConnection,
    factory: &CreateModelStepFactory,
    model: &domain::Model,
) -> Result<domain::ProviderConnection, Error> {
    factory.call(model.provider_id.clone())
        .context("Capability blocked: no worker protocol for this provider; choose a supported model with /model")
        .map_err(Error::caused_by)?;
    let policy = policy
        .call()
        .await
        .context("Allowlist configuration could not be read; open /model settings")
        .map_err(Error::caused_by)?;
    if !policy.permits(&model.provider_id, &model.id) {
        return Err::<domain::ProviderConnection, _>(Box::<dyn std::error::Error + Send + Sync>::from("Model identity is outside the positive allowlist"))
            .context("Allowlist blocked: this provider/model is not permitted; edit the positive patterns in /model settings")
            .map_err(Error::caused_by);
    }
    connection.call(model.provider_id.clone()).await
        .context("Authentication blocked: sign in with /auth; catalog presence does not establish account access")
        .map_err(Error::caused_by)
}
