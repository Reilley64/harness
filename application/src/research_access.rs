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
    if child.agent_id != id
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
    if let Some(work_id) = &child.work_id {
        let work = work
            .call(work_id.clone())
            .await?
            .ok_or("Approved Research work is unavailable")?;
        let approval = work.authorize(
            child
                .work_intent
                .as_deref()
                .ok_or("Research approval intent is missing")?,
            child
                .work_version
                .as_deref()
                .ok_or("Research work version is missing")?,
        )?;
        if approval.coordinator_id != child.parent_id
            || Some(approval.approved_at) != child.approved_at
        {
            return Err(
                "Research approval differs from the exact accepted parent authority".into(),
            );
        }
    }
    Ok(())
}
