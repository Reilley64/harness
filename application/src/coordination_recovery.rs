use crate::ports::ReadCoordination;
use crate::ports::UpdateCoordination;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) async fn pause(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    if !conversation.resume_required
        || conversation.agent.as_ref().is_none_or(|agent| {
            !matches!(
                agent.role,
                domain::AgentRole::Griller | domain::AgentRole::ChiefOfStaff
            )
        })
    {
        return Ok(());
    }
    let Some(state) = read.call(conversation.id.clone()).await? else {
        return Ok(());
    };
    if matches!(
        state.gate,
        domain::CoordinationGate::ResumeRequired | domain::CoordinationGate::Stopped
    ) {
        return Ok(());
    }
    if let (Some(controller), Some(run_id)) = (state.controller.clone(), state.run_id.clone()) {
        update
            .call(
                conversation.id.clone(),
                domain::CoordinationCommand::Gate {
                    gate: domain::CoordinationGate::ResumeRequired,
                    controller: Some(controller),
                    run_id: Some(run_id),
                },
                Some(state.revision),
                CancellationToken::new(),
            )
            .await?;
    }
    Ok(())
}

pub(crate) async fn resume(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    let Some(state) = read.call(conversation.id.clone()).await? else {
        return Ok(());
    };
    if state.gate == domain::CoordinationGate::Stopped {
        return Err("Stopped execution cannot Resume; inspect and acknowledge the interruption before a later human instruction".into());
    }
    if state.gate != domain::CoordinationGate::ResumeRequired {
        return Ok(());
    }
    let run = conversation
        .runs
        .last()
        .ok_or("Recovered parent run is missing")?;
    let agent = conversation
        .agent
        .as_ref()
        .ok_or("Recovered parent agent is missing")?;
    update
        .call(
            conversation.id.clone(),
            domain::CoordinationCommand::ResumeParent {
                agent_id: agent.id.0.clone(),
                controller: state
                    .controller
                    .clone()
                    .ok_or("Recovered parent has no controlling identity")?,
                run_id: run.id.clone(),
            },
            Some(state.revision),
            CancellationToken::new(),
        )
        .await?;
    Ok(())
}

pub(crate) async fn acknowledge(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    let Some(state) = read.call(conversation.id.clone()).await? else {
        return Ok(());
    };
    if !matches!(
        state.gate,
        domain::CoordinationGate::ResumeRequired | domain::CoordinationGate::Stopped
    ) || state.controller.is_none()
    {
        return Ok(());
    }
    let run = conversation
        .runs
        .last()
        .ok_or("Recovered parent run is missing")?;
    let agent = conversation
        .agent
        .as_ref()
        .ok_or("Recovered parent agent is missing")?;
    update
        .call(
            conversation.id.clone(),
            domain::CoordinationCommand::AcknowledgeParent {
                agent_id: agent.id.0.clone(),
                controller: state
                    .controller
                    .clone()
                    .ok_or("Recovered parent has no controlling identity")?,
                run_id: run.id.clone(),
            },
            Some(state.revision),
            CancellationToken::new(),
        )
        .await?;
    Ok(())
}
