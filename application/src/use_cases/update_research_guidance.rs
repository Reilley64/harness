mod dependencies;
mod error;
mod output;
pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::work_access;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    parent_id: String,
    revision: u64,
    intent: String,
    snapshot: String,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into().worker;
    if d.read_execution_claim
        .call(intent.clone(), cancellation.clone())
        .await
        .map_err(Error::caused_by)?
        .is_some()
    {
        return Err(Error::caused_by(
            "Accepted assignments retain their immutable guidance; inspect them instead of adopting changed instructions",
        ));
    }
    let parent = d
        .read_conversation
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    let agent = parent
        .agent
        .as_ref()
        .filter(|a| {
            matches!(
                a.role,
                domain::AgentRole::Griller | domain::AgentRole::ChiefOfStaff
            )
        })
        .ok_or_else(|| Error::caused_by("Research adoption requires the actual parent"))?;
    if parent.revision != revision || parent.resume_required {
        return Err(Error::caused_by(
            "Parent changed or requires Resume; inspect again",
        ));
    }
    work_access::read(&d.read_work, &parent_id, parent.assignment.as_ref())
        .await
        .map_err(Error::caused_by)?;
    let workspace = parent
        .workspace
        .clone()
        .ok_or_else(|| Error::caused_by("Parent workspace is missing"))?;
    let available = d
        .read_instructions
        .call(workspace, "research".into(), Vec::new(), Vec::new())
        .await
        .map_err(Error::caused_by)?
        .available
        .ok_or_else(|| Error::caused_by("Required Research guidance is missing"))?;
    if available.id != snapshot {
        return Err(Error::caused_by(
            "Available Research instructions changed; inspect again before adoption",
        ));
    }
    let guidance = domain::ResearchGuidance {
        intent,
        parent_id: parent_id.clone(),
        parent_agent_id: agent.id.0.clone(),
        work_id: parent.assignment.as_ref().map(|w| w.work_id.clone()),
        work_intent: parent.assignment.as_ref().map(|w| w.intent.clone()),
        work_version: parent
            .assignment
            .as_ref()
            .map(|w| w.proposal.version.clone()),
        snapshot: available,
    };
    let current = d
        .read_conversation
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    if current.revision != revision || !guidance.matches(&current) {
        return Err(Error::caused_by(
            "Parent changed before Research adoption; inspect again",
        ));
    }
    work_access::read(&d.read_work, &parent_id, current.assignment.as_ref())
        .await
        .map_err(Error::caused_by)?;
    let guidance = d
        .update_research_guidance
        .call(guidance, cancellation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { guidance })
}
