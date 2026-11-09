use crate::ports;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn read(
    coordination: &ports::ReadCoordination,
    work: &ports::ReadWork,
    research: Option<&domain::ResearchAssignment>,
    id: &str,
    model: &domain::Model,
    effort: Option<domain::ModelEffort>,
    connection_id: &str,
    instructions: Option<&str>,
) -> Result<(), Error> {
    let Some(research) = research else {
        return Ok(());
    };
    let parent = coordination
        .call(research.parent_id.clone())
        .await?
        .ok_or("Research coordination is missing; inspect the accepted execution without replay")?;
    if !matches!(
        parent.gate,
        domain::CoordinationGate::Open | domain::CoordinationGate::NeedsHuman
    ) {
        return Err(
            "Research parent is gated; retain results without new inference or effects".into(),
        );
    }
    let child = parent
        .children
        .iter()
        .find(|child| child.intent == research.intent)
        .ok_or("Research execution claim is missing")?;
    if child.stop_requested
        || child.resume_required
        || child.parent_agent_id != research.parent_agent_id
        || child.agent_id != id
        || child.assignment_id != research.assignment_id
        || child.execution_id != research.execution_id
        || child.status != domain::ChildExecutionStatus::Running
        || child.selected.selection.provider_id != model.provider_id
        || child.selected.selection.model_id != model.id
        || child.selected.selection.effort != effort
        || child.selected.connection_id != connection_id
        || child.selected.model.canonical_id != model.canonical_id
        || Some(child.instructions.id.as_str()) != instructions
    {
        return Err("Research selection, execution or captured guidance changed; inspect without replacement".into());
    }
    model.validate_context(child.brief.context_tokens)?;
    authorize(work, child).await
}

pub(crate) async fn monitor(
    coordination: &ports::ReadCoordination,
    research: domain::ResearchAssignment,
) -> Result<(), Error> {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let state = coordination
            .call(research.parent_id.clone())
            .await?
            .ok_or("Research ownership disappeared; inspect without replay")?;
        let child = state
            .children
            .iter()
            .find(|child| child.execution_id == research.execution_id)
            .ok_or("Research execution disappeared; inspect without replay")?;
        match state.execution_gate(&child.execution_id)? {
            domain::CoordinationGate::Stopped => return Ok(()),
            domain::CoordinationGate::Open | domain::CoordinationGate::NeedsHuman => {}
            _ => {
                return Err(
                    "Research ownership is gated for recovery or inspection, not stopped".into(),
                );
            }
        }
    }
}

pub(crate) async fn authorize(
    work: &ports::ReadWork,
    child: &domain::ChildExecution,
) -> Result<(), Error> {
    if let Some(work_id) = &child.work_id {
        let work = work
            .call(work_id.clone())
            .await?
            .ok_or("Approved Research work is unavailable")?;
        child.authorize_work(&work)?;
    }
    Ok(())
}
