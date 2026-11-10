use crate::conversations::StorageError;
use crate::repository;
use application::ports::AgentIndexResponse;
use application::ports::PortFuture;
use std::path::Path;

pub fn read_agent_index(
    directory: &Path,
) -> impl Fn(
    std::path::PathBuf,
    Option<String>,
    tokio_util::sync::CancellationToken,
) -> PortFuture<Result<AgentIndexResponse, StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |workspace, after, cancellation_token| {
        let directory = directory.clone();
        Box::pin(async move {
            if let Some(after) = &after {
                let id = uuid::Uuid::parse_str(after)?;
                if id.to_string() != *after {
                    return Err("Invalid creation-map cursor".into());
                }
            }
            let (workspace, _) = repository::identity(&workspace).await?;
            let mut entries = match tokio::fs::read_dir(&directory).await {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(AgentIndexResponse {
                        workspace,
                        conversations: Vec::new(),
                        next: None,
                    });
                }
                Err(error) => return Err(error.into()),
            };

            let mut ids = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                if cancellation_token.is_cancelled() {
                    return Err("Creation-map read cancelled".into());
                }
                let name = entry.file_name();
                let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
                    continue;
                };
                let Ok(uuid) = uuid::Uuid::parse_str(id) else {
                    continue;
                };
                if uuid.to_string() != id
                    || after.as_ref().is_some_and(|after| id <= after.as_str())
                {
                    continue;
                }
                if !entry.file_type().await?.is_file() {
                    return Err("Unsafe conversation index entry".into());
                }
                let at = ids.partition_point(|prior: &String| prior.as_str() < id);
                ids.insert(at, id.to_owned());
                ids.truncate(65);
            }
            let next = if ids.len() > 64 {
                ids.truncate(64);
                ids.last().cloned()
            } else {
                None
            };

            Ok(AgentIndexResponse {
                workspace,
                conversations: ids,
                next,
            })
        })
    }
}
