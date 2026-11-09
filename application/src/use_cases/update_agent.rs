use crate::coordination_recovery;

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
    conversation.recover();
    coordination_recovery::pause(
        &dependencies.read_coordination,
        &dependencies.update_coordination,
        &conversation,
    )
    .await
    .map_err(Error::caused_by)?;
    conversation = coordination_recovery::restore(
        &dependencies.read_coordination,
        &dependencies.update_coordination,
        &dependencies.lock_conversation,
        &dependencies.lock_execution,
        &dependencies.read_conversation,
        &dependencies.update_conversation,
        conversation,
        &tokio_util::sync::CancellationToken::new(),
    )
    .await
    .map_err(Error::caused_by)?;
    conversation
        .acknowledge_interruption()
        .map_err(Error::caused_by)?;
    coordination_recovery::acknowledge(
        &dependencies.read_coordination,
        &dependencies.update_coordination,
        &conversation,
    )
    .await
    .map_err(Error::caused_by)?;
    let conversation = dependencies
        .update_conversation
        .call(conversation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { conversation })
}
