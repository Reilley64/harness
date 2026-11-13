mod idle;
pub(crate) use idle::restore;

use crate::ports::LockExecution;
use crate::ports::ReadCoordination;
use crate::ports::UpdateConversation;
use crate::ports::UpdateCoordination;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) async fn restore_implementation(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    execution: &LockExecution,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    if !conversation.resume_required
        || conversation
            .agent
            .as_ref()
            .is_none_or(|agent| agent.role != domain::AgentRole::Implementer)
    {
        return Ok(());
    }

    let owner = conversation
        .research
        .as_ref()
        .ok_or("Interrupted Implementer has no execution owner")?;
    let _lease = match execution.call(owner.execution_id.clone()).await {
        Ok(lease) => lease,
        Err(error) if error.busy => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    let state = read
        .call(owner.parent_id.clone())
        .await?
        .ok_or("Interrupted Implementer coordination is missing")?;
    let child = state
        .children
        .iter()
        .find(|child| child.execution_id == owner.execution_id)
        .ok_or("Interrupted Implementer execution is missing")?;
    if child.agent_id != conversation.id
        || child.assignment_id != owner.assignment_id
        || child.parent_agent_id != owner.parent_agent_id
        || child.intent != owner.intent
    {
        return Err("Interrupted Implementer differs from its durable execution owner".into());
    }

    if !child.stop_requested
        && child.outcome.is_none()
        && matches!(
            child.status,
            domain::ChildExecutionStatus::Held | domain::ChildExecutionStatus::Running
        )
    {
        update
            .call(
                owner.parent_id.clone(),
                domain::CoordinationCommand::RecoverChild {
                    execution_id: owner.execution_id.clone(),
                    notice_id: uuid::Uuid::new_v4().to_string(),
                },
                None,
                CancellationToken::new(),
            )
            .await?;
    }

    Ok(())
}

pub(crate) async fn restore_answer(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    save: &UpdateConversation,
    conversation: domain::Conversation,
) -> Result<domain::Conversation, Error> {
    let Some(research) = &conversation.research else {
        return Ok(conversation);
    };
    if conversation.runs.last().is_none_or(|run| {
        !matches!(
            run.status,
            domain::RunStatus::Running | domain::RunStatus::Interrupted
        )
    }) {
        return Ok(conversation);
    }
    let owner = read
        .call(research.parent_id.clone())
        .await?
        .ok_or("Pending answer recovery lost its execution ownership")?;
    let notice = conversation
        .runs
        .last()
        .ok_or("Pending answer run is missing")?
        .id
        .clone();
    let (conversation, changed) = tokio::task::spawn_blocking(move || {
        let mut conversation = conversation;
        let changed = conversation.restore_pending_input(&owner)?;
        Ok::<_, Error>((conversation, changed))
    })
    .await??;
    if !changed {
        return Ok(conversation);
    }

    if conversation.resume_required {
        let own = read
            .call(conversation.id.clone())
            .await?
            .ok_or("Interrupted answer recovery lost its own coordination")?;
        update
            .call(
                conversation.id.clone(),
                domain::CoordinationCommand::RetainRecovery {
                    agent_id: conversation
                        .agent
                        .as_ref()
                        .ok_or("Pending input has no durable agent")?
                        .id
                        .0
                        .clone(),
                    notices: vec![notice],
                    idle: true,
                },
                Some(own.revision),
                CancellationToken::new(),
            )
            .await?;
    }
    Ok(save.call(conversation).await?)
}

pub(crate) async fn pause(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    if !conversation.resume_required
        || conversation.agent.as_ref().is_none_or(|agent| {
            !matches!(
                agent.role,
                domain::AgentRole::Griller
                    | domain::AgentRole::ChiefOfStaff
                    | domain::AgentRole::Research
                    | domain::AgentRole::Implementer
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
        let state = update
            .call(
                conversation.id.clone(),
                domain::CoordinationCommand::RetainRecovery {
                    agent_id: state.agent_id.clone(),
                    notices: vec![uuid::Uuid::new_v4().to_string()],
                    idle: false,
                },
                Some(state.revision),
                CancellationToken::new(),
            )
            .await?
            .coordination;
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

pub(crate) async fn resume_input(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    conversation: &domain::Conversation,
) -> Result<(), Error> {
    let Some(state) = read.call(conversation.id.clone()).await? else {
        return Ok(());
    };
    if state.restart_required {
        update
            .call(
                conversation.id.clone(),
                domain::CoordinationCommand::ResumeIdle {
                    agent_id: state.agent_id.clone(),
                },
                Some(state.revision),
                CancellationToken::new(),
            )
            .await?;
        return Ok(());
    }
    if state.gate != domain::CoordinationGate::ResumeRequired {
        return Err("Recovered input cannot bypass its exact Resume gate or Stop".into());
    }
    update
        .call(
            conversation.id.clone(),
            domain::CoordinationCommand::ResumeInputParent {
                agent_id: conversation
                    .agent
                    .as_ref()
                    .ok_or("Recovered input has no durable agent")?
                    .id
                    .0
                    .clone(),
                controller: state
                    .controller
                    .clone()
                    .ok_or("Recovered input has no controlling identity")?,
                run_id: conversation
                    .runs
                    .last()
                    .ok_or("Recovered input has no durable run")?
                    .id
                    .clone(),
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
    if state.restart_required {
        update
            .call(
                conversation.id.clone(),
                domain::CoordinationCommand::ResumeIdle {
                    agent_id: state.agent_id.clone(),
                },
                Some(state.revision),
                CancellationToken::new(),
            )
            .await?;
        return Ok(());
    }
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
