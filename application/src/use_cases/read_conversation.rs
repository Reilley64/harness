mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

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
    if conversation.recover() {
        conversation = dependencies
            .update_conversation
            .call(conversation)
            .await
            .map_err(Error::caused_by)?;
    }
    Ok(Output { conversation })
}
