mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    agent_id: String,
    control: domain::ChildrenControl,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    let conversation = d
        .read_conversation
        .call(agent_id.clone())
        .await
        .map_err(Error::caused_by)?;
    if let domain::ChildrenControl::Stop { execution_id } = &control {
        let state = d
            .read_coordination
            .call(agent_id.clone())
            .await
            .map_err(Error::caused_by)?
            .ok_or_else(|| Error::caused_by("Parent coordination is missing"))?;
        let response = d
            .update_coordination
            .call(
                agent_id,
                domain::CoordinationCommand::StopChild {
                    execution_id: execution_id.clone(),
                },
                Some(state.revision),
                cancellation,
            )
            .await
            .map_err(Error::caused_by)?;
        return Ok(Output {
            coordination: response.coordination,
        });
    }
    if conversation.resume_required
        || conversation.agent.as_ref().is_none_or(|agent| {
            !matches!(
                agent.role,
                domain::AgentRole::ChiefOfStaff | domain::AgentRole::Griller
            )
        })
    {
        return Err(Error::caused_by(
            "Child control requires its actual eligible parent; inspect without replay",
        ));
    }
    let run = conversation
        .runs
        .last()
        .filter(|run| run.status == domain::RunStatus::Running)
        .ok_or_else(|| {
            Error::caused_by("Captured joins belong to the current active logical run")
        })?;
    let coordination = d
        .read_coordination
        .call(agent_id.clone())
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("No accepted child executions are available"))?;
    let command = match control {
        domain::ChildrenControl::Stop { execution_id } => {
            domain::CoordinationCommand::StopChild { execution_id }
        }
        domain::ChildrenControl::Join {
            id,
            mode,
            executions,
        } => domain::CoordinationCommand::Join(domain::CapturedJoin {
            id,
            run_id: run.id.clone(),
            mode,
            executions,
            backgrounded: false,
        }),
        domain::ChildrenControl::Background { id } => domain::CoordinationCommand::Background {
            join_id: id,
            run_id: run.id.clone(),
        },
        domain::ChildrenControl::CompleteFollowUp { outcomes }
            if outcomes.is_empty() || outcomes.len() > 256 =>
        {
            return Err(Error::caused_by(
                "Follow-up requires 1..256 exact delivered receipt identities",
            ));
        }
        domain::ChildrenControl::CompleteFollowUp { outcomes } => {
            domain::CoordinationCommand::CompleteFollowUp {
                controller: coordination.controller.clone().ok_or_else(|| {
                    Error::caused_by("No execution controls this parent follow-up")
                })?,
                run_id: run.id.clone(),
                outcomes,
            }
        }
    };
    let response = d
        .update_coordination
        .call(agent_id, command, Some(coordination.revision), cancellation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output {
        coordination: response.coordination,
    })
}
