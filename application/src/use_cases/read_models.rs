mod dependencies;
mod error;
mod output;

use crate::error_context;
use crate::model_access;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(dependencies: impl Into<Dependencies>) -> Result<Vec<Output>, Error> {
    let dependencies = dependencies.into();
    let models = dependencies
        .read_models
        .call()
        .await
        .map_err(Error::caused_by)?;
    let mut output = Vec::with_capacity(models.len());
    for model in models {
        let blocked = model_access::read(
            &dependencies.read_model_allowlist,
            &dependencies.read_provider_connection,
            &dependencies.create_model_step_factory,
            &model,
        )
        .await
        .err()
        .map(|error| error_context::diagnostic(&error));
        let mut summary = Output::from(model);
        summary.blocked = blocked;
        output.push(summary);
    }
    Ok(output)
}
