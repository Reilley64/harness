use crate::coordination_recovery;

mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    workspace: std::path::PathBuf,
    role: Option<domain::AgentRole>,
) -> Result<Output, Error> {
    let dependencies = dependencies.into();
    let role = match role {
        Some(role) => role,
        None => dependencies
            .read_setup
            .call(
                workspace.clone(),
                tokio_util::sync::CancellationToken::new(),
            )
            .await
            .map_err(Error::caused_by)?
            .role(),
    };
    let Some(id) = dependencies
        .read_agent
        .call(workspace, role.clone())
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
        .is_none_or(|agent| agent.role != role)
    {
        return Err(Error::caused_by(
            "Repository association does not match the requested role",
        ));
    }
    if conversation.recover() {
        coordination_recovery::pause(
            &dependencies.read_coordination,
            &dependencies.update_coordination,
            &conversation,
        )
        .await
        .map_err(Error::caused_by)?;
        conversation = dependencies
            .update_conversation
            .call(conversation)
            .await
            .map_err(Error::caused_by)?;
    }
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
    Ok(Output {
        conversation: Some(conversation),
    })
}
