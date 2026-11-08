mod dependencies;
mod error;
mod output;

use crate::decision_access;
use crate::error_context::diagnostic;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(dependencies: impl Into<Dependencies>, id: String) -> Result<Output, Error> {
    let d = dependencies.into();
    let decision = d.read_decision.call(id).await.map_err(Error::caused_by)?;
    let mut blocked = None;
    if let Some(decision) = &decision {
        if decision.phase == domain::DecisionPhase::Configuration {
            blocked = decision.reason.clone();
        } else if let Some(selected) = &decision.selected {
            match decision_access::candidate(&d.read_models, &d.read_model_allowlist, &d.read_provider_connection, &d.create_model_step_factory, &selected.selection, decision.brief.context_tokens).await {
                Ok(current) if current.connection_id == selected.connection_id && current.model.canonical_id == selected.model.canonical_id => {}
                Ok(_) => blocked = Some("Selected connection or canonical revision changed; configure without replacement".into()),
                Err(error) => blocked = Some(diagnostic(error.as_ref())),
            }
        }
    }
    Ok(Output { decision, blocked })
}
