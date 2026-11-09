mod record;
use self::record::Record;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn read_research_guidance(
    directory: &Path,
) -> impl Fn(String) -> PortFuture<Result<Option<domain::ResearchGuidance>, Error>> + Send + Sync + use<>
{
    let directory = directory.to_owned();
    move |intent| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&intent)?;
            let path = directory.join(format!("{intent}.json"));
            match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
            }
            storage::private_directory(&directory).await?;
            let bytes = storage::read_private(&path, 2 * 1024 * 1024).await?;
            tokio::task::spawn_blocking(move || {
                let guidance: domain::ResearchGuidance =
                    serde_json::from_slice::<Record>(&bytes)?.try_into()?;
                if guidance.intent != intent {
                    return Err("Research guidance key differs from record".into());
                }
                Ok(Some(guidance))
            })
            .await?
        })
    }
}
