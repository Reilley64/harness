use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    read_agent: ports::ReadAgent,
    read_conversation: ports::ReadConversation,
    update_conversation: ports::UpdateConversation,
    lock_conversation: ports::LockConversation,
}

#[ohno::error]
#[derive(Default)]
#[display("Could not read repository Griller")]
pub struct Error;

pub struct Output {
    pub conversation: Option<domain::Conversation>,
}

pub async fn call(
    dependencies: impl Into<Dependencies>,
    workspace: std::path::PathBuf,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let Some(id) = dependencies
        .read_agent
        .call(workspace)
        .await
        .map_err(Error::caused_by)?
    else {
        return Ok(Output { conversation: None });
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
    Ok(Output {
        conversation: Some(conversation),
    })
}
