mod record;
use self::record::Record;
use application::error_context::ResultExt;
use std::path::Path;

use application::ports::PortFuture;

use crate::conversations;

pub fn read_conversation(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<domain::Conversation, conversations::StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |id| {
        let directory = directory.clone();
        Box::pin(async move {
            let bytes = conversations::read(&directory, &id).await?;
            tokio::task::spawn_blocking(move || {
                let record: Record =
                    serde_json::from_slice(&bytes).context("Corrupt conversation record")?;
                let conversation: domain::Conversation =
                    record.try_into().context("Invalid stored conversation")?;
                if conversation.id != id {
                    return Err("Conversation identity does not match its file".into());
                }
                Ok(conversation)
            })
            .await
            .context("Could not process conversation record")?
        })
    }
}
