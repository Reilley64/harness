mod record;
use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;
type Error = Box<dyn std::error::Error + Send + Sync>;
pub fn update_work(
    directory: &Path,
) -> impl Fn(domain::ApprovedWork, Option<u64>) -> PortFuture<Result<domain::ApprovedWork, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |mut work, revision| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&work.id)?;
            let path = directory.join(format!("{}.json", work.id));
            let _guard = storage::lock(&path).await?;
            let bytes = match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => Some(storage::read_private(&path, 4 * 1024 * 1024).await?),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.into()),
            };
            let (work, encoded) = tokio::task::spawn_blocking(move || {
                let existing: Option<domain::ApprovedWork> = bytes
                    .map(|b| serde_json::from_slice::<Record>(&b))
                    .transpose()?
                    .map(TryInto::try_into)
                    .transpose()?;
                if existing.as_ref().map(|w| w.revision) != revision
                    || work.revision != revision.unwrap_or(0)
                {
                    return Err::<_, Error>(
                        "Stale approved-work revision; reload before retrying".into(),
                    );
                }
                if let Some(prior) = existing {
                    if work.id != prior.id
                        || work.creator_id != prior.creator_id
                        || work.conversation_id != prior.conversation_id
                        || work.workspace != prior.workspace
                        || !work.publications.starts_with(&prior.publications)
                        || work.approvals.len() < prior.approvals.len()
                    {
                        return Err("Immutable approved-work facts changed".into());
                    }
                    for (old, new) in prior.approvals.iter().zip(&work.approvals) {
                        let mut expected = old.clone();
                        expected.delivered = new.delivered;
                        if expected != *new || old.delivered && !new.delivered {
                            return Err("Immutable approval facts changed".into());
                        }
                    }
                }
                work.revision = work
                    .revision
                    .checked_add(1)
                    .ok_or("Work revision exhausted")?;
                let record = Record::from(work.clone());
                let checked: domain::ApprovedWork =
                    serde_json::from_slice::<Record>(&encode_bounded(&record, 4 * 1024 * 1024)?)?
                        .try_into()?;
                let encoded = encode_bounded(&Record::from(checked), 4 * 1024 * 1024)?;
                Ok((work, encoded))
            })
            .await??;
            storage::replace(&path, &encoded).await?;
            Ok(work)
        })
    }
}
