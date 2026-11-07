use crate::error_context::ResultExt;

mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    revision: u64,
    available: String,
    phase: String,
    scopes: Vec<String>,
    references: Vec<String>,
) -> Result<Output, Error> {
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
    if conversation.revision != revision {
        return Err("Stale revision")
            .context("Conversation changed; inspect instructions again before adoption")
            .map_err(Error::caused_by);
    }
    if conversation.agent.is_none() {
        return Err("Legacy assignment").context("Legacy conversations have no role assignment; historical instruction provenance is unknown").map_err(Error::caused_by);
    }
    let workspace = conversation
        .workspace
        .clone()
        .ok_or_else(|| Error::caused_by("The role has no workspace"))?;
    let resolution = dependencies
        .read_instructions
        .call(workspace, phase, scopes, references)
        .await
        .map_err(Error::caused_by)?;
    let snapshot = resolution
        .available
        .ok_or("Required instructions unavailable")
        .context(
            "Required instructions are blocked; inspect /guidance for dependencies and recovery",
        )
        .map_err(Error::caused_by)?;
    if snapshot.id != available {
        return Err("Stale instructions")
            .context("Available instructions changed; inspect the new sources before adoption")
            .map_err(Error::caused_by);
    }
    conversation
        .adopt_instructions(snapshot)
        .context("Instruction adoption requires idle work and a valid snapshot; interrupted work retains its captured instructions")
        .map_err(Error::caused_by)?;
    let conversation = dependencies
        .update_conversation
        .call(conversation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { conversation })
}
