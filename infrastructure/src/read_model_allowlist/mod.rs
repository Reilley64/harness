use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type StorageError = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    patterns: Vec<String>,
    #[serde(rename = "customized")]
    _customized: bool,
    initialized_providers: Vec<String>,
}

pub fn read_model_allowlist(
    path: &Path,
) -> impl Fn() -> PortFuture<Result<domain::ModelAllowlist, StorageError>> + Send + Sync + use<> {
    let path = path.to_owned();
    move || {
        let path = path.clone();
        Box::pin(async move {
            storage::private_directory(
                path.parent()
                    .ok_or("Model allowlists require a parent directory")?,
            )
            .await?;
            let bytes = match storage::read_private(&path, 262_144).await {
                Ok(bytes) => bytes,
                Err(error)
                    if error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound) =>
                {
                    return Ok(domain::ModelAllowlist::default());
                }
                Err(error) => return Err(error),
            };
            tokio::task::spawn_blocking(move || {
                let record: Record = serde_json::from_slice(&bytes)?;
                if record.version != 1 || record.initialized_providers.len() > 256 {
                    return Err("Invalid or unsupported model allowlist".into());
                }
                Ok(domain::ModelAllowlist::new(record.patterns)?)
            })
            .await?
        })
    }
}
