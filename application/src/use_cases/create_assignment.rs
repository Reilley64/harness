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
    assignment: Option<&domain::WorkAssignment>,
    parent: domain::ModelSelection,
    parent_connection_id: String,
    parent_revision: u64,
    parent_run_id: Option<String>,
    brief: domain::AssignmentBrief,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    brief.validate().map_err(Error::caused_by)?;
    if !uuid::Uuid::parse_str(&brief.intent).is_ok_and(|v| v.to_string() == brief.intent) {
        return Err(Error::caused_by(
            "Assignment intent must be a canonical UUID",
        ));
    }
    let agent = sender_agent
        .filter(|a| a.role == domain::AgentRole::ChiefOfStaff)
        .ok_or_else(|| {
            Error::caused_by("Assignment preparation requires the actual Chief of Staff")
        })?;
    let assignment =
        assignment.ok_or_else(|| Error::caused_by("Approved assignment authority is missing"))?;
    let work = d
        .read_work
        .call(assignment.work_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Approved work is missing"))?;
    work.authorize_assignment(assignment, sender_id)
        .map_err(Error::caused_by)?;

    let decision = domain::AssignmentDecision {
        id: brief.intent.clone(),
        parent_id: sender_id.into(),
        parent_agent_id: agent.id.0.clone(),
        parent_revision,
        parent_run_id,
        parent,
        parent_connection_id,
        work_id: assignment.work_id.clone(),
        work_intent: assignment.intent.clone(),
        work_version: assignment.proposal.version.clone(),
        brief,
        legacy: false,
        revision: 0,
        finished_at: None,
        phase: domain::DecisionPhase::Prepared,
        request: None,
        response: None,
        stages: Vec::new(),
        selected: None,
        reason: None,
    };
    if let Some(prior) = d
        .read_decision
        .call(decision.id.clone())
        .await
        .map_err(Error::caused_by)?
    {
        if !prior.same_intent(&decision) {
            return Err(Error::caused_by(
                "Assignment intent is bound to a different exact payload",
            ));
        }
        return Ok(Output { decision: prior });
    }
    if cancellation.is_cancelled() {
        return Err(Error::caused_by(
            "Assignment preparation stopped before admission",
        ));
    }

    let decision = d
        .update_decision
        .call(decision, None)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output { decision })
}
