use super::Error;
use crate::ports::LockConversation;
use crate::ports::LockExecution;
use crate::ports::PortFuture;
use crate::ports::ReadConversation;
use crate::ports::ReadCoordination;
use crate::ports::UpdateConversation;
use crate::ports::UpdateCoordination;
use tokio_util::sync::CancellationToken;

type Recovery = (domain::Conversation, Vec<String>, usize);

pub(crate) async fn restore(
    read: &ReadCoordination,
    update: &UpdateCoordination,
    lock: &LockConversation,
    execution: &LockExecution,
    conversations: &ReadConversation,
    save: &UpdateConversation,
    conversation: domain::Conversation,
    cancellation: &CancellationToken,
) -> Result<domain::Conversation, Error> {
    let (conversation, _, _) = walk(
        read.clone(),
        update.clone(),
        lock.clone(),
        execution.clone(),
        conversations.clone(),
        save.clone(),
        conversation,
        Vec::new(),
        4096,
        None,
        cancellation.clone(),
    )
    .await?;
    Ok(conversation)
}

fn walk(
    read: ReadCoordination,
    update: UpdateCoordination,
    lock: LockConversation,
    execution: LockExecution,
    conversations: ReadConversation,
    save: UpdateConversation,
    mut conversation: domain::Conversation,
    mut visited: Vec<String>,
    mut remaining: usize,
    forced_notice: Option<String>,
    cancellation: CancellationToken,
) -> PortFuture<Result<Recovery, Error>> {
    Box::pin(async move {
        if cancellation.is_cancelled() {
            return Err("Execution recovery inspection was cancelled".into());
        }
        if remaining == 0 || visited.len() >= 64 || visited.contains(&conversation.id) {
            return Err("Recovery ownership is cyclic or exceeds its bounded walk; inspect narrower owned scopes without inference".into());
        }
        remaining -= 1;
        visited.push(conversation.id.clone());
        if forced_notice.is_some() && !conversation.resume_required {
            conversation.resume_required = true;
            conversation = save.call(conversation).await?;
        }
        let Some(state) = read.call(conversation.id.clone()).await? else {
            return Ok((conversation, forced_notice.into_iter().collect(), remaining));
        };
        if !state.authority_complete {
            return Err(
                "Required recovery ownership is incomplete; no empty work set was inferred".into(),
            );
        }
        let mut notices = if conversation.resume_required {
            state.recovery_notices
        } else {
            Vec::new()
        };
        notices.extend(forced_notice);

        for child in state.children {
            if cancellation.is_cancelled() {
                return Err("Execution recovery inspection was cancelled".into());
            }
            if child.outcome.is_none()
                && let Some(notice) = &child.recovery_notice
            {
                notices.push(notice.clone());
            }
            let _owner = match execution.call(child.execution_id.clone()).await {
                Ok(owner) => owner,
                Err(error) if error.busy => continue,
                Err(error) => return Err(error.into()),
            };
            let _guard = lock.call(child.agent_id.clone()).await?;
            let child_source = conversations.call(child.agent_id.clone()).await?;
            if child_source.id != child.agent_id
                || child_source.research.as_ref().is_none_or(|research| {
                    research.parent_id != conversation.id
                        || research.parent_agent_id != child.parent_agent_id
                        || research.intent != child.intent
                        || research.assignment_id != child.assignment_id
                        || research.execution_id != child.execution_id
                })
            {
                return Err(
                    "Recovered child source differs from its retained execution ownership".into(),
                );
            }
            let (mut child_source, changed) = tokio::task::spawn_blocking(move || {
                let mut source = child_source;
                let changed = source.recover();
                (source, changed)
            })
            .await?;
            if changed {
                super::pause(&read, &update, &child_source).await?;
                child_source = save.call(child_source).await?;
            }
            let mut notice = None;
            if child.outcome.is_none()
                && (matches!(
                    child.status,
                    domain::ChildExecutionStatus::Held | domain::ChildExecutionStatus::Running
                ) || child.resume_required)
            {
                let result = update
                    .call(
                        conversation.id.clone(),
                        domain::CoordinationCommand::RecoverChild {
                            execution_id: child.execution_id.clone(),
                            notice_id: uuid::Uuid::new_v4().to_string(),
                        },
                        None,
                        cancellation.clone(),
                    )
                    .await?;
                let recovered = result
                    .coordination
                    .children
                    .iter()
                    .find(|entry| entry.execution_id == child.execution_id)
                    .ok_or("Recovered child disappeared from ownership")?;
                if let Some(id) = &recovered.recovery_notice {
                    notices.push(id.clone());
                    if recovered.resume_required {
                        notice = Some(id.clone());
                    }
                }
            }
            let (source, descendants, next_remaining) = walk(
                read.clone(),
                update.clone(),
                lock.clone(),
                execution.clone(),
                conversations.clone(),
                save.clone(),
                child_source,
                visited.clone(),
                remaining,
                notice,
                cancellation.clone(),
            )
            .await?;
            drop(source);
            remaining = next_remaining;
            notices.extend(descendants);
        }
        notices.sort();
        notices.dedup();
        if !notices.is_empty() {
            let agent_id = conversation
                .agent
                .as_ref()
                .ok_or("Recovery has no durable agent identity")?
                .id
                .0
                .clone();
            let idle = conversation
                .runs
                .last()
                .is_none_or(|run| run.status != domain::RunStatus::Interrupted);
            let retained = update
                .call(
                    conversation.id.clone(),
                    domain::CoordinationCommand::RetainRecovery {
                        agent_id,
                        notices,
                        idle,
                    },
                    None,
                    cancellation,
                )
                .await?
                .coordination;
            if retained.restart_required && !conversation.resume_required {
                conversation.resume_required = true;
                conversation = save.call(conversation).await?;
            }
            let pending = if conversation.resume_required {
                retained.recovery_notices
            } else {
                Vec::new()
            };
            return Ok((conversation, pending, remaining));
        }
        Ok((conversation, Vec::new(), remaining))
    })
}
