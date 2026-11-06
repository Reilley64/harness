use std::error::Error;
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use application::error_context::ResultExt;
use application::ports::PortFuture;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::storage;

type FileError = Box<dyn Error + Send + Sync>;
const CATALOG_LIMIT: usize = 16 * 1024 * 1024;

struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
}

impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(
                "Serialized data exceeds its size limit",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn encode_bounded<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>, FileError> {
    let mut writer = BoundedWriter {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer_pretty(&mut writer, value)?;
    Ok(writer.bytes)
}

pub(crate) fn json_string_len(value: &str) -> usize {
    value.bytes().fold(2usize, |size, byte| {
        size.saturating_add(match byte {
            b'"' | b'\\' | b'\n' | b'\r' | b'\t' | 8 | 12 => 2,
            0..=31 => 6,
            _ => 1,
        })
    })
}

pub(crate) async fn remote_json<T: DeserializeOwned + Send + 'static>(
    client: reqwest::Client,
    url: String,
) -> Result<T, FileError> {
    let mut response = client
        .get(url)
        .send()
        .await
        .context("Could not fetch catalog")?
        .error_for_status()
        .context("Catalog request was rejected")?;
    if response
        .content_length()
        .is_some_and(|size| size > CATALOG_LIMIT as u64)
    {
        return Err("Catalog exceeds its size limit".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.context("Could not read catalog")? {
        if chunk.len() > CATALOG_LIMIT.saturating_sub(bytes.len()) {
            return Err("Catalog exceeds its size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    tokio::task::spawn_blocking(move || {
        serde_json::from_slice(&bytes).context("Invalid catalog response")
    })
    .await
    .context("Could not process catalog")?
    .map_err(Into::into)
}

pub fn with_memory_cache<T, E, F, Fut>(
    read: F,
    ttl: Duration,
) -> impl Fn() -> PortFuture<Result<T, E>> + Send + Sync
where
    T: Clone + Send + Sync + 'static,
    E: From<tokio::task::JoinError> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
{
    let read = Arc::new(read);
    let memory = Arc::new(Mutex::new(None::<(Instant, Arc<T>)>));
    move || {
        let read = Arc::clone(&read);
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut cached = memory.lock().await;
            if let Some((loaded, value)) = cached.as_ref()
                && loaded.elapsed() < ttl
            {
                let value = Arc::clone(value);
                return Ok(tokio::task::spawn_blocking(move || value.as_ref().clone()).await?);
            }
            let value = Arc::new(read().await?);
            let copy = Arc::clone(&value);
            let result = tokio::task::spawn_blocking(move || copy.as_ref().clone()).await?;
            *cached = Some((Instant::now(), value));
            Ok(result)
        })
    }
}

pub fn with_file_fallback<T, E, F, Fut>(
    read: F,
    path: &Path,
) -> impl Fn() -> PortFuture<Result<T, FileFallbackError>> + Send + Sync + use<T, E, F, Fut>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
    E: Into<FileError> + Send + Sync + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
{
    let read = Arc::new(read);
    let path = Arc::new(path.to_path_buf());
    move || {
        let read = Arc::clone(&read);
        let path = Arc::clone(&path);
        Box::pin(async move {
            match read().await {
                Ok(value) => {
                    let (value, encoded) = tokio::task::spawn_blocking(move || {
                        let encoded = encode_bounded(&value, CATALOG_LIMIT);
                        (value, encoded)
                    })
                    .await?;
                    let saved = async {
                        let bytes = encoded?;
                        let parent = path.parent().ok_or("Cache requires a parent directory")?;
                        storage::private_directory(parent).await?;
                        storage::replace(path.as_ref(), &bytes).await
                    }
                    .await;
                    if saved.is_err() {
                        tracing::warn!("Could not persist catalog cache");
                    }
                    Ok(value)
                }
                Err(read_error) => {
                    let loaded = async {
                        let bytes = storage::read_private(path.as_ref(), CATALOG_LIMIT).await?;
                        tokio::task::spawn_blocking(move || {
                            serde_json::from_slice(&bytes).context("Invalid catalog cache")
                        })
                        .await?
                        .map_err(FileError::from)
                    }
                    .await;
                    loaded.map_err(|disk_error| {
                        FileFallbackError::caused_by(read_error.into(), disk_error)
                    })
                }
            }
        })
    }
}

#[ohno::error]
#[display("Catalog source and fallback are unavailable")]
pub struct FileFallbackError {
    pub source_error: FileError,
}

impl From<tokio::task::JoinError> for FileFallbackError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::caused_by("Catalog processing failed", error)
    }
}
