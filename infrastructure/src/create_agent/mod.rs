mod record;
use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::repository;
use application::error_context::ResultExt;
use application::ports::PortFuture;
use std::path::Path;

pub fn create_agent(
    directory: &Path,
) -> impl Fn(
    domain::Model,
    Option<domain::ModelEffort>,
    domain::ProviderConnection,
    std::path::PathBuf,
) -> PortFuture<Result<String, conversations::StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |model, effort, connection, workspace| {
        let directory = directory.clone();
        Box::pin(async move {
            let (workspace, id) = repository::identity(&workspace).await?;
            let _guard = conversations::lock(&directory, &id).await?;
            match tokio::fs::symlink_metadata(directory.join(format!("{id}.json"))).await {
                Ok(_) => return Ok(id),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            if model.provider_id != connection.provider_id {
                return Err("Model and provider connection do not match".into());
            }
            let now = conversations::now()?;
            let record_id = id.clone();
            let bytes = tokio::task::spawn_blocking(move || {
                let record = Record::new(
                    record_id,
                    uuid::Uuid::new_v4().to_string(),
                    model,
                    effort,
                    connection.id,
                    workspace,
                    now,
                );
                encode_bounded(&record, 16 * 1024 * 1024)
            })
            .await
            .context("Could not encode Griller")??;
            conversations::write(&directory, &id, &bytes).await?;
            Ok(id)
        })
    }
}
