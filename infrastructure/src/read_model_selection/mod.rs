use crate::storage;

use std::path::Path;

use application::error_context::ResultExt;
use application::ports::PortFuture;
use tokio::io::AsyncReadExt;

mod record;
use self::record::Record;

type StorageError = Box<dyn std::error::Error + Send + Sync>;

pub fn read_model_selection(
    path: &Path,
) -> impl Fn() -> PortFuture<Result<Option<domain::SelectedModel>, StorageError>> + Send + Sync + use<>
{
    let path = path.to_owned();
    move || {
        let path = path.clone();
        Box::pin(async move {
            let metadata = match tokio::fs::symlink_metadata(&path).await {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
            };
            storage::private_directory(
                path.parent()
                    .ok_or("Model selections require a parent directory")?,
            )
            .await?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > 1_048_576
            {
                return Err(
                    "Invalid or oversized saved model selection; choose a model with /model".into(),
                );
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o077 != 0 {
                    return Err(
                        "Saved model selections require owner-only permissions (0600)".into(),
                    );
                }
            }
            let mut bytes = Vec::new();
            tokio::fs::File::open(&path)
                .await?
                .take(1_048_577)
                .read_to_end(&mut bytes)
                .await?;
            if bytes.len() > 1_048_576 {
                return Err("Saved model selection exceeds its size limit".into());
            }
            let dto: Record = serde_json::from_slice(&bytes)
                .context("The saved model selection is invalid; choose a model with /model")?;
            Ok(Some(dto.try_into()?))
        })
    }
}
