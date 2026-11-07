mod record;
use self::record::Record;
use std::path::Path;

use application::error_context::ResultExt;
use application::ports::PortFuture;

use crate::cache::encode_bounded;

use crate::conversations;

pub fn create_conversation(
    directory: &Path,
) -> impl Fn(
    domain::Model,
    Option<domain::ModelEffort>,
    domain::ProviderConnection,
    std::path::PathBuf,
) -> PortFuture<Result<domain::Conversation, conversations::StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |model, effort, connection, workspace| {
        let directory = directory.clone();
        Box::pin(async move {
            if model.provider_id != connection.provider_id {
                return Err("Model and provider connection do not match".into());
            }
            let id = uuid::Uuid::new_v4().to_string();
            let _guard = conversations::lock(&directory, &id).await?;
            let now = conversations::now()?;
            let conversation = domain::Conversation {
                id,
                agent: None,
                runs: Vec::new(),
                instruction_snapshots: Vec::new(),
                selected_instructions: None,
                resume_required: false,
                provider_connection_id: connection.id,
                workspace: Some(tokio::fs::canonicalize(workspace).await?),
                model,
                effort,
                messages: Vec::new(),
                created_at: now,
                updated_at: now,
                revision: 0,
            };
            let (conversation, bytes) = tokio::task::spawn_blocking(move || {
                let bytes = encode_bounded(&Record::from(conversation.clone()), 16 * 1024 * 1024)
                    .context("Conversation exceeds its size limit")?;
                Ok::<_, conversations::StorageError>((conversation, bytes))
            })
            .await
            .context("Could not encode conversation")??;
            conversations::write(&directory, &conversation.id, &bytes).await?;
            Ok(conversation)
        })
    }
}
