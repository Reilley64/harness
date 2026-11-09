mod record;
use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;
use tokio_util::sync::CancellationToken;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn update_research_guidance(
    directory: &Path,
) -> impl Fn(
    domain::ResearchGuidance,
    CancellationToken,
) -> PortFuture<Result<domain::ResearchGuidance, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |guidance, cancellation| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&guidance.intent)?;
            storage::private_directory(&directory).await?;
            let intents = directory
                .parent()
                .ok_or("Research guidance storage has no owner")?
                .join("intents");
            storage::private_directory(&intents).await?;
            let guard = storage::lock(&intents.join(format!("{}.lock", guidance.intent))).await?;
            match tokio::fs::symlink_metadata(intents.join(format!("{}.json",guidance.intent))).await {
                Ok(_)=>return Err("Accepted executions retain their captured instructions; adopt for a new intent".into()),
                Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{},
                Err(error)=>return Err(error.into()),
            }
            let path = directory.join(format!("{}.json", guidance.intent));
            let previous = match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => Some(storage::read_private(&path, 2 * 1024 * 1024).await?),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.into()),
            };
            let (guidance,bytes)=tokio::task::spawn_blocking(move||{
                guidance.validate()?;
                if let Some(previous)=previous{
                    let existing:domain::ResearchGuidance=serde_json::from_slice::<Record>(&previous)?.try_into()?;
                    if existing!=guidance{return Err::<_,Error>("An intent already adopted a different immutable Research closure; inspect it or use an intentional new intent".into());}
                    return Ok((existing,None));
                }
                let bytes=encode_bounded(&Record::from(guidance.clone()),2*1024*1024)?;
                Ok((guidance,Some(bytes)))
            }).await??;
            if cancellation.is_cancelled() {
                return Err("Research guidance adoption stopped before commit".into());
            }
            if let Some(bytes) = bytes {
                storage::replace(&path, &bytes).await?;
            }
            drop(guard);
            Ok(guidance)
        })
    }
}
