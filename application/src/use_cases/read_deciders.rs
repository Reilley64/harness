mod dependencies;
mod error;
mod output;

use crate::decision_access;
use crate::error_context::diagnostic;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(dependencies: impl Into<Dependencies>) -> Result<Output, Error> {
    let d = dependencies.into();
    let selected = d.read_decider.call().await.map_err(Error::caused_by)?;
    let mut models = Vec::new();
    let mut diagnostics = Vec::new();
    for model in d.read_models.call().await.map_err(Error::caused_by)? {
        if model.validate_decider().is_err() {
            continue;
        }
        let blocked = match decision_access::native(
            &d.read_models,
            &d.read_model_allowlist,
            &d.read_provider_connection,
            &model.id,
        )
        .await
        {
            Ok(_) => None,
            Err(error) => {
                let blocked = diagnostic(error.as_ref());
                diagnostics.push(Error::caused_by(error));
                Some(blocked)
            }
        };
        models.push((model, blocked));
    }
    Ok(Output {
        models,
        selected,
        diagnostics,
    })
}
