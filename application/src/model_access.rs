use crate::error_context::ResultExt;
use crate::ports::CreateModelStepFactory;
use crate::ports::ReadModelAllowlist;
use crate::ports::ReadProviderConnection;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("Model configuration required")]
pub(crate) struct Error;

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
