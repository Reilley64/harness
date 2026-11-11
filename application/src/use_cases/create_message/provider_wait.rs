use super::Dependencies;
use super::Error;
use super::accept;
use super::checkpoint;
use super::managed::Lease;
use crate::ports::PresentTurnEvent;
use crate::turn_control::SteeringRequest;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio_util::sync::CancellationToken;

pub(super) fn now() -> Result<u64, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into())
        .map_err(Error::caused_by)?
        .map_err(Error::caused_by)
}

pub(super) async fn wait(
    dependencies: &Dependencies,
    conversation: &mut domain::Conversation,
    buffer: &Arc<Mutex<String>>,
    steering: &mut tokio::sync::mpsc::Receiver<SteeringRequest>,
    cancellation: &CancellationToken,
) -> Result<bool, Error> {
    loop {
        let Some(retry) = &conversation.provider_retry else {
            return Ok(true);
        };
        if !retry.waiting {
            return Ok(true);
        }
        let until = retry.retry_at;
        let current = now()?;
        if cancellation.is_cancelled() {
            return Ok(false);
        }
        if until.is_some_and(|until| until <= current) {
            return Ok(true);
        }

        tokio::select! {
            _ = cancellation.cancelled() => return Ok(false),
            Some(request) = steering.recv() => {
                accept(dependencies, conversation, request, buffer, cancellation).await?;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(until.map_or(100, |until| until.saturating_sub(current).min(100)))) => {
                let stored = dependencies.read_conversation.call(conversation.id.clone()).await.map_err(Error::caused_by)?;
                if stored.revision > conversation.revision {
                    if stored.messages.last().map(|reply| &reply.id) != conversation.messages.last().map(|reply| &reply.id) {
                        return Err(Error::caused_by("Provider wait controller ownership changed"));
                    }
                    *conversation = stored;
                }
                super::instructions::synchronize(dependencies, conversation, buffer, cancellation).await?;
            }
        }
    }
}

pub(super) async fn failed(
    dependencies: &Dependencies,
    conversation: &mut domain::Conversation,
    lease: Option<&Lease>,
    failure: domain::ProviderFailure,
    present: &PresentTurnEvent,
) -> Result<(), Error> {
    let initial_allowance = if conversation.provider_retry.is_none() {
        initial_allowance(dependencies, conversation).await?
    } else {
        domain::ProviderRetry::DEFAULT_ALLOWANCE
    };

    let retry = domain::ProviderRetry::failed(
        conversation.provider_retry.clone(),
        conversation
            .active_reply()
            .map_err(Error::caused_by)?
            .id
            .clone(),
        failure,
        initial_allowance,
        now()?,
        uuid::Uuid::new_v4().as_u128() as u64,
    )
    .map_err(Error::caused_by)?;
    conversation.provider_retry = Some(retry);

    checkpoint(dependencies, conversation).await?;
    super::managed::suspend(dependencies, conversation, lease, &CancellationToken::new()).await?;

    present
        .call(domain::TurnEvent::Usage {
            usage: conversation.usage(),
        })
        .map_err(Error::caused_by)
}

async fn initial_allowance(
    dependencies: &Dependencies,
    conversation: &domain::Conversation,
) -> Result<u32, Error> {
    if let Some(assignment) = &conversation.assignment {
        let work = dependencies
            .read_work
            .call(assignment.work_id.clone())
            .await
            .map_err(Error::caused_by)?
            .ok_or_else(|| Error::caused_by("Approved retry work is unavailable"))?;

        work.authorize_assignment(assignment, &conversation.id)
            .map_err(Error::caused_by)?;
        let approval = work
            .authorize(&assignment.intent, &assignment.proposal.version)
            .map_err(Error::caused_by)?;
        return Ok(approval
            .transient_retry_allowance
            .unwrap_or(domain::ProviderRetry::DEFAULT_ALLOWANCE));
    }

    if let Some(research) = &conversation.research {
        let state = dependencies
            .read_coordination
            .call(research.parent_id.clone())
            .await
            .map_err(Error::caused_by)?
            .ok_or_else(|| Error::caused_by("Retry execution ownership is unavailable"))?;

        let child = state
            .children
            .iter()
            .find(|child| child.execution_id == research.execution_id)
            .ok_or_else(|| Error::caused_by("Retry execution ownership is missing"))?;
        if !state.authority_complete
            || state.agent_id != research.parent_agent_id
            || child.agent_id != conversation.id
            || child.assignment_id != research.assignment_id
            || child.intent != research.intent
        {
            return Err(Error::caused_by("Retry execution authority changed"));
        }
        if let Some(work_id) = &child.work_id {
            let work = dependencies
                .read_work
                .call(work_id.clone())
                .await
                .map_err(Error::caused_by)?
                .ok_or_else(|| Error::caused_by("Approved retry work is unavailable"))?;

            child.authorize_work(&work).map_err(Error::caused_by)?;
            let approval = work
                .authorize(
                    child
                        .work_intent
                        .as_deref()
                        .ok_or_else(|| Error::caused_by("Retry approval intent is missing"))?,
                    child
                        .work_version
                        .as_deref()
                        .ok_or_else(|| Error::caused_by("Retry approval version is missing"))?,
                )
                .map_err(Error::caused_by)?;
            return Ok(approval
                .transient_retry_allowance
                .unwrap_or(domain::ProviderRetry::DEFAULT_ALLOWANCE));
        }
    }

    Ok(domain::ProviderRetry::DEFAULT_ALLOWANCE)
}
