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
            let _mutation = conversations::lock_mutation(&directory, &id).await?;
            let now = conversations::now()?;
            let workspace = Some(tokio::fs::canonicalize(workspace).await?);
            let conversation = domain::Conversation {
                id,
                agent: None,
                assignment: None,
                research: None,

                workspace,
                provider_connection_id: connection.id,
                model,
                effort,

                instruction_snapshots: Vec::new(),
                selected_instructions: None,

                messages: Vec::new(),
                runs: Vec::new(),
                current_work: None,
                resume_required: false,
                provider_retry: None,
                provider_retry_history: Vec::new(),

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
