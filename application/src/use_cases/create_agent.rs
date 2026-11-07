mod dependencies;
mod error;
mod output;

use crate::model_access;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    selection: domain::ModelSelection,
    workspace: std::path::PathBuf,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let existing = dependencies
        .read_agent
        .call(workspace.clone())
        .await
        .map_err(Error::caused_by)?;
    let id = if let Some(id) = existing {
        id
    } else {
        let model = model_access::read_model(
            &dependencies.read_models,
            &selection.provider_id,
            &selection.model_id,
            selection.effort,
        )
        .await
        .map_err(Error::caused_by)?;
        let connection = model_access::read(
            &dependencies.read_model_allowlist,
            &dependencies.read_provider_connection,
            &dependencies.create_model_step_factory,
            &model,
        )
        .await
        .map_err(Error::caused_by)?;
        dependencies
            .create_agent
            .call(model, selection.effort, connection, workspace)
            .await
            .map_err(Error::caused_by)?
    };
    let _guard = dependencies
        .lock_conversation
        .call(id.clone())
        .await
        .map_err(Error::caused_by)?;
    let mut conversation = dependencies
        .read_conversation
        .call(id)
        .await
        .map_err(Error::caused_by)?;
    if conversation
        .agent
        .as_ref()
        .is_none_or(|agent| agent.role != domain::AgentRole::Griller)
    {
        return Err(Error::caused_by("Repository association is not a Griller"));
    }
    if conversation.recover() {
        conversation = dependencies
            .update_conversation
            .call(conversation)
            .await
            .map_err(Error::caused_by)?;
    }
    Ok(Output { conversation })
}
