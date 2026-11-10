mod record;

use self::record::Record;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn read_decision(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<Option<domain::AssignmentDecision>, Error>> + Send + Sync + use<>
{
    let directory = directory.to_owned();
    move |id| {
        let path = directory.join(format!("{id}.json"));
        Box::pin(async move {
            conversations::validate_id(&id)?;
            match tokio::fs::symlink_metadata(&path).await {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
                Ok(_) => {}
            }
            let bytes = storage::read_private(&path, 64 * 1024 * 1024).await?;
            tokio::task::spawn_blocking(move || {
                let r: Record = serde_json::from_slice(&bytes)?;
                let d: domain::AssignmentDecision = r.try_into()?;
                if d.id != id {
                    return Err("Decision identity mismatch".into());
                }
                Ok(Some(d))
            })
            .await?
        })
    }
}
