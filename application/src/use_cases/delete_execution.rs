mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::use_cases::read_children;
use std::collections::HashSet;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    conversation_id: String,
    execution_id: Option<String>,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    if cancellation.is_cancelled() {
        return Err(Error::caused_by(
            "Stop was cancelled before durable admission",
        ));
    }
    let (state, mut diagnostics) = read_children::load(&d.read_children, conversation_id.clone())
        .await
        .map_err(Error::caused_by)?;
    let result = async {
        let (state, execution_id) = match (state, execution_id) {
            (Some(state), execution_id) => (state, execution_id),
            (None, None) => {
                let conversation = d.read_children.read_conversation.call(conversation_id).await.map_err(Error::caused_by)?;
                let research = conversation.research.ok_or_else(|| Error::caused_by("Execution ownership is not yet admitted; cancel the pending local prompt or inspect its durable claims"))?;
                let (owner, recovered) = read_children::load(&d.read_children, research.parent_id).await.map_err(Error::caused_by)?;
                diagnostics.extend(recovered);
                let owner = owner.ok_or_else(|| Error::caused_by("Research execution owner is missing"))?;
                if !owner.children.iter().any(|child| child.intent == research.intent
                    && child.assignment_id == research.assignment_id
                    && child.execution_id == research.execution_id
                    && child.agent_id == conversation.id
                    && child.parent_agent_id == research.parent_agent_id) {
                    return Err(Error::caused_by("Research lineage does not match its accepted owner"));
                }
                (owner, Some(research.execution_id))
            }
            (None, Some(_)) => return Err(Error::caused_by("Selected parent execution ownership is missing")),
        };
        stop(&d, state, execution_id, &mut diagnostics).await
    }.await;
    match result {
        Ok(output) if diagnostics.is_empty() => Ok(output),
        Ok(_) => {
            let original = diagnostics.remove(0);
            Err(error::retain(Error::caused_by(original), diagnostics))
        }
        Err(original) => Err(error::retain(original, diagnostics)),
    }
}

async fn stop(
    d: &Dependencies,
    state: domain::Coordination,
    execution_id: Option<String>,
    diagnostics: &mut Vec<Box<dyn std::error::Error + Send + Sync>>,
) -> Result<Output, Error> {
    let mut executions = Vec::new();
    let mut pending = Vec::new();
    if !state.authority_complete {
        diagnostics.push(
            "Stop fenced known ownership, but missing admission facts require explicit inspection"
                .into(),
        );
    }
    let coordination = if let Some(execution_id) = execution_id {
        let child = state
            .children
            .iter()
            .find(|child| child.execution_id == execution_id)
            .ok_or_else(|| Error::caused_by("Execution is not owned by the selected parent"))?;
        pending.push((child.agent_id.clone(), child.agent_id.clone()));
        let response = d
            .read_children
            .update_coordination
            .call(
                state.conversation_id,
                domain::CoordinationCommand::StopChild {
                    execution_id: execution_id.clone(),
                },
                Some(state.revision),
                CancellationToken::new(),
            )
            .await
            .map_err(Error::caused_by)?;
        d.child_runtime.cancel(&execution_id);
        executions.push(execution_id);
        response.coordination
    } else {
        let conversation = d
            .read_children
            .read_conversation
            .call(state.conversation_id.clone())
            .await
            .map_err(Error::caused_by)?;
        if conversation.agent.as_ref().map(|agent| agent.id.0.as_str())
            != Some(state.agent_id.as_str())
        {
            return Err(Error::caused_by(
                "Selected conversation and execution owner differ",
            ));
        }
        if let Some(research) = conversation.research {
            let (owner, recovered) =
                read_children::load(&d.read_children, research.parent_id.clone())
                    .await
                    .map_err(Error::caused_by)?;
            diagnostics.extend(recovered);
            let owner =
                owner.ok_or_else(|| Error::caused_by("Research execution owner is missing"))?;
            let child = owner
                .children
                .iter()
                .find(|child| {
                    child.intent == research.intent
                        && child.assignment_id == research.assignment_id
                        && child.execution_id == research.execution_id
                        && child.agent_id == conversation.id
                        && child.parent_agent_id == research.parent_agent_id
                })
                .ok_or_else(|| {
                    Error::caused_by("Research lineage does not match its accepted owner")
                })?;
            let execution_id = child.execution_id.clone();
            d.read_children
                .update_coordination
                .call(
                    owner.conversation_id,
                    domain::CoordinationCommand::StopChild {
                        execution_id: execution_id.clone(),
                    },
                    Some(owner.revision),
                    CancellationToken::new(),
                )
                .await
                .map_err(Error::caused_by)?;
            d.child_runtime.cancel(&execution_id);
            executions.push(execution_id);
        }
        fence(d, state, &mut pending, &mut executions).await?
    };

    let mut visited = HashSet::from([coordination.conversation_id.clone()]);
    while let Some((id, agent_id)) = pending.pop() {
        if !visited.insert(id.clone()) {
            diagnostics.push(
                "Execution ownership contains a cycle or multiple owners; inspect retained lineage"
                    .into(),
            );
            continue;
        }
        let result = async {
            let (state, recovered) = read_children::load(&d.read_children, id).await.map_err(Error::caused_by)?;
            diagnostics.extend(recovered);
            let Some(state) = state else { return Ok(()) };
            if state.agent_id != agent_id {
                return Err(Error::caused_by("Descendant coordination belongs to a different durable agent"));
            }
            if !state.authority_complete {
                diagnostics.push("Descendant Stop fenced known ownership, but missing admission facts require explicit inspection".into());
            }
            fence(d, state, &mut pending, &mut executions).await?;
            Ok::<_, Error>(())
        }.await;
        if let Err(failure) = result {
            diagnostics.push(Box::new(failure));
        }
    }
    Ok(Output {
        coordination,
        executions,
    })
}

async fn fence(
    d: &Dependencies,
    state: domain::Coordination,
    pending: &mut Vec<(String, String)>,
    executions: &mut Vec<String>,
) -> Result<domain::Coordination, Error> {
    let response = d
        .read_children
        .update_coordination
        .call(
            state.conversation_id,
            domain::CoordinationCommand::Stop {
                agent_id: state.agent_id,
            },
            Some(state.revision),
            CancellationToken::new(),
        )
        .await
        .map_err(Error::caused_by)?;
    for child in &response.coordination.children {
        d.child_runtime.cancel(&child.execution_id);
        executions.push(child.execution_id.clone());
        pending.push((child.agent_id.clone(), child.agent_id.clone()));
    }
    Ok(response.coordination)
}
