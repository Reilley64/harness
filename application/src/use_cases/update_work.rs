mod dependencies;
mod error;
mod output;

use crate::error_context::ResultExt;
use crate::model_access;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    revision: u64,
    version: String,
    intent: String,
    selection: domain::ModelSelection,
    transient_retry_allowance: Option<u32>,
    session_revision: u64,
) -> Result<Output, Error> {
    let d = dependencies.into();
    let namespace = uuid::Uuid::parse_str(&id).map_err(Error::caused_by)?;
    if namespace.to_string() != id
        || !uuid::Uuid::parse_str(&intent).is_ok_and(|v| v.to_string() == intent)
    {
        return Err(Error::caused_by(
            "Work and assignment intents must be canonical UUIDs",
        ));
    }

    if transient_retry_allowance
        .is_some_and(|allowance| allowance > domain::ProviderRetry::MAX_INITIAL_ALLOWANCE)
    {
        return Err(Error::caused_by(
            "Initial transient retry allowance must be within 0..64",
        ));
    }

    let mut work = d
        .read_work
        .call(id)
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Published work was not found"))?;
    let coordinator_id = uuid::Uuid::new_v5(&namespace, intent.as_bytes()).to_string();
    let prior = work.approvals.iter().find(|a| a.intent == intent).cloned();
    if let Some(prior) = &prior
        && (prior.version != version
            || prior.revision != revision
            || prior.selection != selection
            || prior.transient_retry_allowance != transient_retry_allowance)
    {
        return Err("Assignment intent is bound to a different approval payload").context("Assignment intent conflict: retry the exact original payload or explicitly choose a distinct intent").map_err(Error::caused_by);
    }
    work.current().map_err(Error::caused_by)?;
    if work.current().map_err(Error::caused_by)?.version != version {
        return Err("Approval version is no longer current").context("Approval is stale; inspect the current proposal before approving its exact scope/criteria version").map_err(Error::caused_by);
    }

    let model = model_access::read_model(
        &d.read_models,
        &selection.provider_id,
        &selection.model_id,
        selection.effort,
    )
    .await
    .map_err(Error::caused_by)?;
    let connection = model_access::read(
        &d.read_model_allowlist,
        &d.read_provider_connection,
        &d.create_model_step_factory,
        &model,
    )
    .await
    .map_err(Error::caused_by)?;

    if prior.is_none() {
        let guard = d
            .lock_conversation
            .call(work.conversation_id.clone())
            .await
            .map_err(Error::caused_by)?;
        let sender = d
            .read_conversation
            .call(work.conversation_id.clone())
            .await
            .map_err(Error::caused_by)?;
        if sender.revision != session_revision
            || sender.resume_required
            || sender
                .messages
                .iter()
                .any(|m| m.status == domain::MessageStatus::InProgress)
            || sender.model.provider_id != selection.provider_id
            || sender.model.id != selection.model_id
            || sender.effort != selection.effort
        {
            return Err("Griller target/revision/model changed or work is interrupted").context("Stale approval target/revision/model or interrupted work; inspect again before approval").map_err(Error::caused_by);
        }
        if sender
            .agent
            .as_ref()
            .is_none_or(|a| a.id.0 != work.creator_id || a.role != domain::AgentRole::Griller)
        {
            return Err(Error::caused_by("Work creator association changed"));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(Error::caused_by)?
            .as_secs();
        let expected = work.revision;
        work.approve(domain::WorkApproval {
            intent: intent.clone(),
            version: version.clone(),
            revision,
            selection,
            transient_retry_allowance,
            coordinator_id: coordinator_id.clone(),
            approved_at: now,
            delivered: false,
        })
        .context("Approval conflict or stale work revision; reload the exact proposal and retain the original assignment intent")
        .map_err(Error::caused_by)?;
        work = d
            .update_work
            .call(work, Some(expected))
            .await
            .map_err(Error::caused_by)?;
        drop(guard);
    }

    let approval = work
        .authorize(&intent, &version)
        .map_err(Error::caused_by)?
        .clone();
    if approval.delivered {
        return Ok(Output {
            work,
            conversation_id: coordinator_id,
        });
    }

    let resolution = d
        .read_instructions
        .call(
            work.workspace.clone(),
            "coordination".into(),
            Vec::new(),
            Vec::new(),
        )
        .await
        .map_err(Error::caused_by)?;
    let instructions=resolution.available.ok_or("Required CoS guidance is unavailable").context("Approval retained; required CoS guidance is unavailable. Inspect /guidance phase coordination, repair configuration, then explicitly retry this original intent").map_err(Error::caused_by)?;
    let conversation_id = d
        .create_coordinator
        .call(work.clone(), approval, model, connection, instructions, resolution.overrides.is_empty())
        .await
        .context("Approval retained but handoff could not be confirmed; inspect work and explicitly retry the original intent. No inference was dispatched")
        .map_err(Error::caused_by)?;

    let expected = work.revision;
    work.approvals
        .iter_mut()
        .find(|a| a.intent == intent)
        .ok_or_else(|| Error::caused_by("Assignment claim disappeared"))?
        .delivered = true;
    work = d
        .update_work
        .call(work, Some(expected))
        .await
        .map_err(Error::caused_by)?;

    Ok(Output {
        work,
        conversation_id,
    })
}
