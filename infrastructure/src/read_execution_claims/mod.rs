mod record;
use self::record::Record;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;
use tokio_util::sync::CancellationToken;
type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    format: u32,
    intent: String,
    parent_id: String,
    parent_agent_id: String,
}

pub fn read_execution_claims(
    directory: &Path,
) -> impl Fn(String, CancellationToken) -> PortFuture<Result<Vec<domain::ChildExecution>, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |parent, cancellation| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&parent)?;
            let index = directory.join("parents").join(&parent);
            match tokio::fs::symlink_metadata(&index).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
                Err(e) => return Err(e.into()),
            }
            storage::private_directory(&index).await?;
            let mut entries = tokio::fs::read_dir(&index).await?;
            let mut result = Vec::new();
            let mut total = 0usize;
            while let Some(entry) = entries.next_entry().await? {
                if cancellation.is_cancelled() {
                    return Err("Claim discovery stopped".into());
                }
                if result.len() >= 256 {
                    return Err(
                        "Required claim membership exceeds 256; inspect without inference".into(),
                    );
                }
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".tmp"))
                {
                    continue;
                }
                let reference_bytes = storage::read_private(&entry.path(), 4096).await?;
                let reference: Reference =
                    tokio::task::spawn_blocking(move || serde_json::from_slice(&reference_bytes))
                        .await??;
                conversations::validate_id(&reference.intent)?;
                if reference.format != 1
                    || reference.parent_id != parent
                    || entry.file_name() != format!("{}.json", reference.intent).as_str()
                {
                    return Err("Invalid parent claim membership".into());
                }
                let path = directory
                    .join("intents")
                    .join(format!("{}.json", reference.intent));
                let guard = loop {
                    match storage::lock(&path).await {
                        Ok(guard) => break guard,
                        Err(error)
                            if error
                                .source()
                                .and_then(|e| e.downcast_ref::<std::fs::TryLockError>())
                                .is_some_and(|e| {
                                    matches!(e, std::fs::TryLockError::WouldBlock)
                                }) =>
                        {
                            tokio::select! {_=cancellation.cancelled()=>return Err("Claim discovery stopped".into()),_=tokio::time::sleep(std::time::Duration::from_millis(10))=>{}}
                        }
                        Err(error) => return Err(error),
                    }
                };
                let bytes = storage::read_private(&path, 4 * 1024 * 1024).await?;
                total = total.saturating_add(bytes.len());
                if total > 4 * 1024 * 1024 {
                    return Err(
                        "Required discovered claim data exceeds 4 MiB; inspect without inference"
                            .into(),
                    );
                }
                let claim: domain::ChildExecution = tokio::task::spawn_blocking(move || {
                    let _guard = guard;
                    let claim: domain::ChildExecution =
                        serde_json::from_slice::<Record>(&bytes)?.try_into()?;
                    if claim.intent != reference.intent
                        || claim.parent_id != reference.parent_id
                        || claim.parent_agent_id != reference.parent_agent_id
                    {
                        return Err::<_, Error>(
                            "Claim membership differs from its immutable source".into(),
                        );
                    }
                    Ok(claim)
                })
                .await??;
                result.push(claim);
            }
            Ok(result)
        })
    }
}
