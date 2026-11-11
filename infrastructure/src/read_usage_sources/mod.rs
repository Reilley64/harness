use crate::conversations;
use application::ports::PortFuture;
use application::ports::UsageSourcesResponse;
use std::path::Path;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn read_usage_sources(
    app_data_path: &Path,
) -> impl Fn(CancellationToken) -> PortFuture<Result<UsageSourcesResponse, Error>> + Send + Sync + use<>
{
    let directory = app_data_path.to_owned();
    move |cancellation| {
        let directory = directory.clone();
        Box::pin(async move {
            let conversations = ids(&directory.join("conversations"), &cancellation).await?;
            let decisions = ids(&directory.join("decisions"), &cancellation).await?;
            Ok(UsageSourcesResponse {
                conversations,
                decisions,
            })
        })
    }
}

async fn ids(directory: &Path, cancellation: &CancellationToken) -> Result<Vec<String>, Error> {
    match tokio::fs::symlink_metadata(directory).await {
        Ok(metadata) if !metadata.is_dir() => return Err("Unsafe usage source directory".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
        _ => {}
    }
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut ids = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if cancellation.is_cancelled() {
            return Err("Usage query cancelled".into());
        }
        let name = entry.file_name();
        let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
            continue;
        };
        conversations::validate_id(id)?;
        if !entry.file_type().await?.is_file() {
            return Err("Unsafe usage source".into());
        }
        if ids.len() >= 4_096 {
            return Err("Usage source bound reached; required accounting cannot be omitted".into());
        }
        ids.push(id.to_owned());
    }
    ids.sort();
    Ok(ids)
}
