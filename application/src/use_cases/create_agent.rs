use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    create_agent: ports::CreateAgent,
    read_agent: ports::ReadAgent,
    read_conversation: ports::ReadConversation,
    update_conversation: ports::UpdateConversation,
    lock_conversation: ports::LockConversation,
    read_models: ports::ReadModels,
    read_provider_connection: ports::ReadProviderConnection,
}

#[ohno::error]
#[derive(Default)]
#[display("Could not open repository Griller")]
pub struct Error;

pub struct Output {
    pub conversation: domain::Conversation,
}

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
        let model = dependencies
            .read_models
            .call()
            .await
            .map_err(Error::caused_by)?
            .into_iter()
            .find(|model| {
                model.id == selection.model_id && model.provider_id == selection.provider_id
            })
            .ok_or_else(|| Error::caused_by("The selected model is unavailable"))?;
        model
            .validate_effort(selection.effort)
            .map_err(Error::caused_by)?;
        let connection = dependencies
            .read_provider_connection
            .call(selection.provider_id)
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
