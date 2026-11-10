mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::use_cases::create_answer;
use tokio_util::sync::CancellationToken;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    parent_id: String,
    id: String,
    version: String,
    approval_id: String,
    cancellation: CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into().worker;
    for value in [&parent_id, &id, &version, &approval_id] {
        if !uuid::Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == *value) {
            return Err(Error::caused_by(
                "Scope approval identities must be canonical UUIDs",
            ));
        }
    }
    let response = d
        .update_coordination
        .call(
            parent_id.clone(),
            domain::CoordinationCommand::ApproveScope {
                id: id.clone(),
                version: version.clone(),
                approval_id: approval_id.clone(),
            },
            None,
            cancellation,
        )
        .await
        .map_err(Error::caused_by)?;
    let scope = response
        .coordination
        .scopes
        .iter()
        .find(|scope| scope.id == id)
        .cloned()
        .ok_or_else(|| Error::caused_by("Accepted scope approval is missing"))?;
    let child = response
        .coordination
        .children
        .iter()
        .find(|child| child.execution_id == scope.execution_id)
        .ok_or_else(|| Error::caused_by("Approved scope lost its execution owner"))?;
    let mut started = false;
    if child.status == domain::ChildExecutionStatus::NeedsHuman
        && scope.supersedes_run.as_deref().is_some_and(|run| {
            child
                .outcome
                .as_ref()
                .and_then(|outcome| outcome.run_id.as_deref())
                == Some(run)
        })
    {
        let conversation = d
            .read_conversation
            .call(child.agent_id.clone())
            .await
            .map_err(Error::caused_by)?;
        if !conversation.resume_required {
            let workspace = conversation
                .workspace
                .ok_or_else(|| Error::caused_by("Approved child has no retained workspace"))?;
            let worker = create_answer::Dependencies::new(d.clone());
            let agent_id = child.agent_id.clone();
            let token = CancellationToken::new();
            let execution_token = token.clone();
            let operation = Box::pin(async move {
                create_answer::approved_scope(
                    worker,
                    agent_id,
                    id,
                    version,
                    approval_id,
                    workspace,
                    execution_token,
                )
                .await
                .map_err(Error::caused_by)
                .and_then(|output| {
                    if output.tool_errors.is_empty() {
                        Ok(())
                    } else {
                        Err(error::retain(output.tool_errors))
                    }
                })
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
            });
            d.child_runtime
                .start_run(
                    parent_id,
                    scope.execution_id.clone(),
                    scope
                        .approved_by
                        .clone()
                        .ok_or_else(|| Error::caused_by("Scope approval identity is missing"))?,
                    operation,
                    token,
                )
                .map_err(Error::caused_by)?;
            started = true;
        }
    }
    Ok(Output { scope, started })
}
