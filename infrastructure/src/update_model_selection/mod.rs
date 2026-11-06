use crate::storage;

use std::path::Path;

use application::ports::PortFuture;

mod record;
use self::record::Record;

type StorageError = Box<dyn std::error::Error + Send + Sync>;

pub fn update_model_selection(
    path: &Path,
) -> impl Fn(domain::SelectedModel) -> PortFuture<Result<domain::SelectedModel, StorageError>>
+ Send
+ Sync
+ use<> {
    let path = path.to_owned();
    move |selection| {
        let path = path.clone();
        Box::pin(async move {
            selection.model.validate_effort(selection.effort)?;
            storage::private_directory(
                path.parent()
                    .ok_or("Model selections require a parent directory")?,
            )
            .await?;
            match tokio::fs::symlink_metadata(&path).await {
                Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
                    return Err("Invalid saved model selection path".into());
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let bytes = serde_json::to_vec_pretty(&Record::from(selection.clone()))?;
            if bytes.len() > 1_048_576 {
                return Err("Saved model selection exceeds its size limit".into());
            }
            storage::replace(&path, &bytes).await?;
            Ok(selection)
        })
    }
}
