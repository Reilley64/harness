mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

use crate::work_access;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    revision: u64,
    extension_id: String,
    additional: Option<u32>,
) -> Result<Output, Error> {
    if !uuid::Uuid::parse_str(&extension_id).is_ok_and(|id| id.to_string() == extension_id) {
        return Err(Error::blocked(
            "A canonical human retry-control identity is required",
        ));
    }

    let dependencies = dependencies.into();
    let mut conversation = dependencies
        .read_conversation
        .call(id.clone())
        .await
        .map_err(Error::unavailable)?;
    let active = conversation
        .messages
        .last()
        .is_some_and(|reply| reply.status == domain::MessageStatus::InProgress);
    let _guard = if active {
        if additional.is_none()
            || conversation
                .provider_retry
                .as_ref()
                .is_none_or(|retry| !retry.waiting)
        {
            return Err(Error::blocked(
                "Only a checkpointed live provider wait can receive Resume or extension; Stop before abandoning",
            ));
        }
        None
    } else {
        let guard = dependencies
            .lock_conversation
            .call(id.clone())
            .await
            .map_err(Error::caused_by)?;
        conversation = dependencies
            .read_conversation
            .call(id)
            .await
            .map_err(Error::caused_by)?;
        Some(guard)
    };
    if conversation.revision != revision {
        return Err(Error::blocked(
            "Inspect the current provider wait revision before changing its authority",
        ));
    }

    work_access::read(
        &dependencies.read_work,
        &conversation.id,
        conversation.assignment.as_ref(),
    )
    .await
    .map_err(Error::caused_by)?;

    let retry = conversation
        .provider_retry
        .as_mut()
        .ok_or_else(|| Error::blocked("There is no retained provider wait"))?;
    if let Some(additional) = additional {
        if additional == 0 {
            retry.resume().map_err(Error::blocked)?;
        } else {
            retry
                .extend(extension_id, additional)
                .map_err(Error::blocked)?;
        }
    } else {
        if conversation.provider_retry_history.len() >= 1_024 {
            return Err(Error::blocked("Provider retry history is full"));
        }
        conversation.provider_retry_history.push(
            conversation
                .provider_retry
                .take()
                .ok_or_else(|| Error::blocked("Provider wait disappeared"))?,
        );
    }

    let conversation = dependencies
        .update_conversation
        .call(conversation)
        .await
        .map_err(Error::caused_by)?;

    Ok(Output { conversation })
}
