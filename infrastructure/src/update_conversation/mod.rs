mod record;
use self::record::Record;
use application::error_context::ResultExt;
use std::path::Path;

use application::ports::PortFuture;

use crate::cache::encode_bounded;
use crate::conversations;

pub fn update_conversation(
    directory: &Path,
) -> impl Fn(
    domain::Conversation,
) -> PortFuture<Result<domain::Conversation, conversations::StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |mut conversation| {
        let directory = directory.clone();
        Box::pin(async move {
            let _mutation = conversations::lock_mutation(&directory, &conversation.id).await?;

            let bytes = conversations::read(&directory, &conversation.id).await?;
            let (conversation, bytes) = tokio::task::spawn_blocking(move || {
                let record: Record =
                    serde_json::from_slice(&bytes).context("Corrupt conversation record")?;
                let existing: domain::Conversation = record.try_into()
                    .context("Invalid stored conversation")?;
                if existing.id != conversation.id {
                    return Err("Conversation identity does not match its file".into());
                }
                if existing.revision != conversation.revision {
                    return Err("Conversation has changed since it was read".into());
                }
                drop(existing);
                conversation.revision = conversation.revision.checked_add(1)
                    .ok_or("Conversation revision overflow")?;
                conversation.updated_at = conversations::now()?;
                let record = Record::from(conversation);
                let bytes = encode_bounded(&record, 16 * 1024 * 1024)
                    .context("Conversation exceeds its size limit")?;
                if bytes.len().saturating_add(record.finishing_capacity()) > 16 * 1024 * 1024 {
                    return Err("Conversation has insufficient capacity for safe tool-result bookkeeping; start a new conversation".into());
                }
                let conversation: domain::Conversation = record.try_into().context("Invalid updated conversation")?;
                Ok::<_, conversations::StorageError>((conversation, bytes))
            }).await.context("Could not process conversation update")??;
            conversations::write(&directory, &conversation.id, &bytes).await?;
            Ok(conversation)
        })
    }
}
