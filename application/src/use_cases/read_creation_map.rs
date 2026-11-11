mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    workspace: std::path::PathBuf,
    after: Option<String>,
    cancellation_token: tokio_util::sync::CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    let page = d
        .read_agent_index
        .call(workspace, after, cancellation_token.clone())
        .await
        .map_err(Error::caused_by)?;

    let mut agents = Vec::new();
    for id in page.conversations {
        if cancellation_token.is_cancelled() {
            return Err(Error::caused_by("Creation-map read cancelled"));
        }
        let conversation = d
            .read_conversation
            .call(id.clone())
            .await
            .map_err(Error::caused_by)?;
        if conversation.workspace.as_ref() != Some(&page.workspace) || conversation.agent.is_none()
        {
            continue;
        }

        let coordination = d
            .read_coordination
            .call(id)
            .await
            .map_err(Error::caused_by)?;
        if let Some(state) = &coordination {
            for child in &state.children {
                if cancellation_token.is_cancelled() {
                    return Err(Error::caused_by("Creation-map read cancelled"));
                }
                let source = d
                    .read_conversation
                    .call(child.agent_id.clone())
                    .await
                    .map_err(Error::caused_by)?;
                if source.research.as_ref().is_none_or(|research| {
                    research.parent_id != conversation.id
                        || research.execution_id != child.execution_id
                        || research.assignment_id != child.assignment_id
                }) {
                    return Err(Error::caused_by(
                        "Retained child ownership differs from its required source",
                    ));
                }
            }
        }

        let execution = if let Some(research) = &conversation.research {
            let parent = d
                .read_coordination
                .call(research.parent_id.clone())
                .await
                .map_err(Error::caused_by)?
                .ok_or_else(|| {
                    Error::caused_by(
                        "Required child ownership is unavailable; inspect the retained parent",
                    )
                })?;
            if !parent.authority_complete {
                return Err(Error::caused_by(
                    "Required child ownership is incomplete; inspect the retained parent",
                ));
            }
            let execution = parent
                .children
                .iter()
                .find(|child| child.execution_id == research.execution_id)
                .ok_or_else(|| {
                    Error::caused_by(
                        "Required child execution is unavailable; inspect the retained parent",
                    )
                })?;
            if execution.agent_id != conversation.id
                || execution.assignment_id != research.assignment_id
            {
                return Err(Error::caused_by(
                    "Child execution identity differs from its retained conversation",
                ));
            }
            Some(execution.clone())
        } else {
            None
        };
        agents.push((conversation, coordination, execution));
    }

    Ok(Output {
        agents,
        next: page.next,
    })
}
