mod record;
use self::record::Record;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;
type Error = Box<dyn std::error::Error + Send + Sync>;
pub fn read_work(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<Option<domain::ApprovedWork>, Error>> + Send + Sync + use<>
{
    let directory = directory.to_owned();
    move |id| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&id)?;
            let path = directory.join(format!("{id}.json"));
            match tokio::fs::symlink_metadata(&path).await {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
                Ok(_) => {}
            }
            let bytes = storage::read_private(&path, 4 * 1024 * 1024).await?;
            tokio::task::spawn_blocking(move || {
                let r: Record = serde_json::from_slice(&bytes)?;
                let w: domain::ApprovedWork = r.try_into()?;
                if w.id != id {
                    return Err("Work identity mismatch".into());
                }
                Ok(Some(w))
            })
            .await?
        })
    }
}
