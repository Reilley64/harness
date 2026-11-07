use crate::error_context::ResultExt;
use tokio_util::sync::CancellationToken;

mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    sender_id: &str,
    sender_agent: Option<&domain::Agent>,
    workspace: std::path::PathBuf,
    id: String,
    intent: String,
    path: String,
    revision: Option<u64>,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    for id in [&id, &intent] {
        if !uuid::Uuid::parse_str(id).is_ok_and(|v| v.to_string() == *id) {
            return Err(Error::caused_by(
                "Work and intent identities must be canonical UUIDs",
            ));
        }
    }
    let agent = sender_agent
        .filter(|a| a.role == domain::AgentRole::Griller)
        .ok_or_else(|| Error::caused_by("Publication requires the repository Griller"))?;

    let proposal = d
        .read_work_proposal
        .call(workspace.clone(), path, cancellation.clone())
        .await
        .map_err(Error::caused_by)?;
    let mut work = d
        .read_work
        .call(id.clone())
        .await
        .map_err(Error::caused_by)?
        .unwrap_or(domain::ApprovedWork {
            id,
            creator_id: agent.id.0.clone(),
            conversation_id: sender_id.into(),
            workspace,
            revision: 0,
            publications: Vec::new(),
            approvals: Vec::new(),
        });
    if work.creator_id != agent.id.0 || work.conversation_id != sender_id {
        return Err(Error::caused_by("Work belongs to another Griller"));
    }

    let prior = (work.revision != 0).then_some(work.revision);
    if work
        .publish(intent, proposal, revision)
        .context("Publication conflict or stale work revision; reload work, retain the original payload for retries, or use a new intent for an amendment")
        .map_err(Error::caused_by)?
    {
        if cancellation.is_cancelled() {
            return Err(Error::caused_by(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Publication cancelled before committing",
            )));
        }

        work = d
            .update_work
            .call(work, prior)
            .await
            .context("Publication could not be committed; reload approved work before retrying the original intent")
            .map_err(Error::caused_by)?;
    }

    Ok(Output { work })
}
