use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    read_conversation: ports::ReadConversation,
    update_conversation: ports::UpdateConversation,
    lock_conversation: ports::LockConversation,
}

#[ohno::error]
#[derive(Default)]
#[display("Could not acknowledge interruption")]
pub struct Error;

pub struct Output {
    pub conversation: domain::Conversation,
}

pub async fn call(dependencies: impl Into<Dependencies>, id: String) -> Result<Output, Error> {
    let dependencies = dependencies.into();
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
    conversation.recover();
    conversation
        .acknowledge_interruption()
        .map_err(Error::caused_by)?;
    let conversation = dependencies
        .update_conversation
        .call(conversation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { conversation })
}
