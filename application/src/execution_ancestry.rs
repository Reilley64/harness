use crate::ports::ReadConversation;
use crate::ports::ReadCoordination;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) struct Boundary {
    pub parent_id: String,
    pub execution_id: String,
    pub controller: String,
    pub gate: domain::CoordinationGate,
    pub pending_instructions: bool,
    pub scope: Option<domain::ScopeAmendment>,
}

pub(crate) async fn read(
    coordination: &ReadCoordination,
    conversations: &ReadConversation,
    id: &str,
    research: &domain::ResearchAssignment,
) -> Result<Vec<Boundary>, Error> {
    let boundaries = inspect(coordination, conversations, id, research).await?;
    if boundaries.iter().any(|boundary| {
        !matches!(
            boundary.gate,
            domain::CoordinationGate::Open | domain::CoordinationGate::NeedsHuman
        )
    }) {
        return Err("Execution ancestry is stopped, recovered or gated; no effect admitted".into());
    }
    Ok(boundaries)
}

pub(crate) async fn stopped(
    coordination: &ReadCoordination,
    conversations: &ReadConversation,
    id: &str,
    research: &domain::ResearchAssignment,
) -> Result<bool, Error> {
    Ok(inspect(coordination, conversations, id, research)
        .await?
        .iter()
        .any(|boundary| boundary.gate == domain::CoordinationGate::Stopped))
}

async fn inspect(
    coordination: &ReadCoordination,
    conversations: &ReadConversation,
    id: &str,
    research: &domain::ResearchAssignment,
) -> Result<Vec<Boundary>, Error> {
    let mut current_id = id.to_owned();
    let mut owner = research.clone();
    let mut boundaries = Vec::new();
    let mut visited = vec![current_id.clone()];
    loop {
        if boundaries.len() >= 256 || visited.contains(&owner.parent_id) {
            return Err("Execution ownership is cyclic or exceeds bounded ancestry inspection; no facts were omitted".into());
        }
        let state = coordination
            .call(owner.parent_id.clone())
            .await?
            .ok_or("Required execution ownership is missing")?;
        let child = state
            .children
            .iter()
            .find(|child| child.execution_id == owner.execution_id)
            .ok_or("Required ancestor execution is missing")?;
        if !state.authority_complete
            || child.agent_id != current_id
            || child.assignment_id != owner.assignment_id
            || child.intent != owner.intent
            || state.agent_id != owner.parent_agent_id
        {
            return Err("Execution ancestry is stopped, recovered, gated or inconsistent; no effect admitted".into());
        }
        boundaries.push(Boundary {
            parent_id: owner.parent_id.clone(),
            execution_id: owner.execution_id.clone(),
            controller: child.controller.clone(),
            gate: state.execution_gate(&owner.execution_id)?,
            pending_instructions: state.instructions.iter().any(|instruction| {
                instruction.execution_id == owner.execution_id && instruction.applied_run.is_none()
            }),
            scope: state.scope(&owner.execution_id).cloned(),
        });
        visited.push(owner.parent_id.clone());
        let parent = conversations.call(owner.parent_id).await?;
        if parent
            .agent
            .as_ref()
            .is_none_or(|agent| agent.id.0 != state.agent_id)
        {
            return Err("Execution ancestor differs from its durable conversation identity".into());
        }
        let Some(research) = parent.research else {
            break;
        };
        current_id = parent.id;
        owner = research;
    }
    Ok(boundaries)
}
