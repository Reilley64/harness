use crate::storage;

use application::error_context::ResultExt;

use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio::io::AsyncReadExt;

use crate::storage::private_directory;

pub(crate) type StorageError = Box<dyn std::error::Error + Send + Sync>;
const LIMIT: u64 = 16 * 1024 * 1024;

pub(crate) fn validate_id(id: &str) -> Result<(), StorageError> {
    let parsed = uuid::Uuid::parse_str(id).context("Invalid conversation or message ID")?;
    if parsed.to_string() != id {
        return Err("IDs must be canonical lowercase UUIDs".into());
    }
    Ok(())
}

pub(crate) fn now() -> Result<u64, StorageError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub(crate) async fn lock(directory: &Path, id: &str) -> Result<std::fs::File, StorageError> {
    validate_id(id)?;
    private_directory(directory).await?;
    let path = directory.join(format!("{id}.lock"));
    if let Ok(metadata) = tokio::fs::symlink_metadata(&path).await
        && (!metadata.is_file() || metadata.file_type().is_symlink())
    {
        return Err("Invalid conversation lock file".into());
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options.open(path).await?.into_std().await;
    tokio::task::spawn_blocking(move || {
        file.try_lock()
            .context("Conversation is busy in another request or process")?;
        Ok(file)
    })
    .await?
}

pub(crate) async fn lock_mutation(
    directory: &Path,
    id: &str,
) -> Result<std::fs::File, StorageError> {
    validate_id(id)?;

    Ok(
        storage::lock(&directory.join(format!("{id}.mutation.json")))
            .await
            .context("Conversation update is busy; inspect its current revision")?,
    )
}

pub(crate) async fn read(directory: &Path, id: &str) -> Result<Vec<u8>, StorageError> {
    validate_id(id)?;
    let path = directory.join(format!("{id}.json"));
    let metadata = tokio::fs::symlink_metadata(&path).await;
    let context = if metadata
        .as_ref()
        .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    {
        "Conversation was not found"
    } else {
        "Could not read conversation file"
    };
    let metadata = metadata.context(context)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("Conversation files must have owner-only permissions (0600)".into());
        }
    }
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > LIMIT {
        return Err("Invalid or oversized conversation file".into());
    }
    let file = tokio::fs::File::open(path).await?;
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes).await?;
    if bytes.len() as u64 > LIMIT {
        return Err("Conversation exceeds its size limit".into());
    }
    Ok(bytes)
}

pub(crate) async fn write(directory: &Path, id: &str, bytes: &[u8]) -> Result<(), StorageError> {
    validate_id(id)?;
    private_directory(directory).await?;
    if bytes.len() as u64 > LIMIT {
        return Err("Conversation exceeds its size limit".into());
    }
    storage::replace(&directory.join(format!("{id}.json")), bytes).await
}
