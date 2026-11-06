use std::path::Path;

use application::error_context::ResultExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

type StorageError = Box<dyn std::error::Error + Send + Sync>;

#[cfg(unix)]
pub(crate) async fn private_directory(path: &Path) -> Result<(), StorageError> {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .await?;
    let metadata = tokio::fs::symlink_metadata(path).await?;
    if !metadata.is_dir()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err("App data directories must be real owner-only directories (0700)".into());
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) async fn private_directory(_path: &Path) -> Result<(), StorageError> {
    Err("Private app data storage is not implemented on this platform".into())
}

fn validate_private_file(metadata: &std::fs::Metadata) -> Result<(), StorageError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;
        if !metadata.is_file()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.nlink() != 1
        {
            return Err("Private app data files must be regular owner-only files (0600)".into());
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Err("Private app data storage is not implemented on this platform".into())
    }
}

pub(crate) async fn read_private(path: &Path, limit: usize) -> Result<Vec<u8>, StorageError> {
    let mut options = tokio::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    let file = options.open(path).await?;
    let metadata = file.metadata().await?;
    validate_private_file(&metadata)?;
    if metadata.len() > limit as u64 {
        return Err("Private app data file exceeds its size limit".into());
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes).await?;
    if bytes.len() > limit {
        return Err("Private app data file exceeds its size limit".into());
    }
    Ok(bytes)
}

pub(crate) async fn lock(path: &Path) -> Result<std::fs::File, StorageError> {
    let parent = path
        .parent()
        .ok_or("Private storage requires a parent directory")?;
    private_directory(parent).await?;
    let mut options = tokio::fs::OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    let file = options.open(path.with_extension("lock")).await?;
    validate_private_file(&file.metadata().await?)?;
    let file = file.into_std().await;
    tokio::task::spawn_blocking(move || {
        file.try_lock()
            .context("Another authentication attempt is already running")?;
        Ok(file)
    })
    .await?
}

pub(crate) async fn replace(
    path: &Path,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parent = path
        .parent()
        .ok_or("Storage files require a parent directory")?
        .to_owned();
    let temporary_directory = parent.clone();
    let temporary =
        tokio::task::spawn_blocking(move || tempfile::NamedTempFile::new_in(temporary_directory))
            .await??;
    let (file, temporary_path) = temporary.into_parts();
    let mut file = tokio::fs::File::from_std(file);
    file.write_all(bytes).await?;
    file.flush().await?;
    file.sync_all().await?;
    drop(file);
    tokio::fs::rename(&temporary_path, path).await?;
    tokio::task::spawn_blocking(move || std::fs::File::open(parent)?.sync_all()).await??;
    Ok(())
}
