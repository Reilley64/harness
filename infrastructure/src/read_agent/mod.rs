use crate::conversations::StorageError;
use crate::repository;
use application::ports::PortFuture;
use std::path::Path;

pub fn read_agent(
    directory: &Path,
) -> impl Fn(std::path::PathBuf, domain::AgentRole) -> PortFuture<Result<Option<String>, StorageError>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |workspace, role| {
        let directory = directory.clone();
        Box::pin(async move {
            let (_, id) = repository::agent_identity(&workspace, role).await?;
            match tokio::fs::symlink_metadata(directory.join(format!("{id}.json"))).await {
                Ok(_) => Ok(Some(id)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error.into()),
            }
        })
    }
}
