mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::research_access;
use crate::work_access;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    parent_id: String,
    execution_id: String,
    action: domain::WorkspaceAction,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into().worker;
    let state = d
        .read_coordination
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Workspace execution owner is missing"))?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == execution_id)
        .ok_or_else(|| Error::caused_by("Workspace assignment is not owned by this coordinator"))?;
    let inspect = action == domain::WorkspaceAction::Inspect;
    if child.brief.role != "implementer" || !child.routed {
        return Err(Error::caused_by(
            "Workspace operations require an approved routed Implementer",
        ));
    }
    if !inspect
        && (state.restart_required
            || !matches!(
                state
                    .execution_gate(&execution_id)
                    .map_err(Error::caused_by)?,
                domain::CoordinationGate::Open | domain::CoordinationGate::NeedsHuman
            )
            || child.status != domain::ChildExecutionStatus::Running
            || child.stop_requested
            || child.resume_required)
    {
        return Err(Error::caused_by(
            "Workspace effects are gated by Stop, input, scope or explicit Resume",
        ));
    }
    let parent = d
        .read_conversation
        .call(parent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    work_access::read(&d.read_work, &parent_id, parent.assignment.as_ref())
        .await
        .map_err(Error::caused_by)?;
    research_access::authorize(&d.read_work, child)
        .await
        .map_err(Error::caused_by)?;
    let repository = parent
        .workspace
        .ok_or_else(|| Error::caused_by("Coordinator has no saved repository"))?;
    let workspace = d
        .update_workspace
        .call(child.clone(), repository, action, cancellation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { workspace })
}
