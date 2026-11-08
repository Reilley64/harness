mod arguments;
mod record;
pub(crate) use arguments::decode;
pub(crate) use arguments::definition;

use self::record::Record;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn read_coordination(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<Option<domain::Coordination>, Error>> + Send + Sync + use<>
{
    let directory = directory.to_owned();
    move |id| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&id)?;
            let path = directory.join(format!("{id}.json"));
            match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
            }
            storage::private_directory(&directory).await?;
            let bytes = storage::read_private(&path, 4 * 1024 * 1024).await?;
            tokio::task::spawn_blocking(move || {
                let coordination: domain::Coordination =
                    serde_json::from_slice::<Record>(&bytes)?.try_into()?;
                if coordination.agent_id != id {
                    return Err("Coordination identity differs from its file".into());
                }
                Ok(Some(coordination))
            })
            .await?
        })
    }
}
