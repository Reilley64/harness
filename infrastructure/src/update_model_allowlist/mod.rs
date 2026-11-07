use crate::storage;
use application::ports::ModelAllowlistUpdate;
use application::ports::PortFuture;
use std::path::Path;

type StorageError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    patterns: Vec<String>,
    customized: bool,
    initialized_providers: Vec<String>,
}

pub fn update_model_allowlist(
    path: &Path,
) -> impl Fn(ModelAllowlistUpdate) -> PortFuture<Result<domain::ModelAllowlist, StorageError>>
+ Send
+ Sync
+ use<> {
    let path = path.to_owned();
    move |update| {
        let path = path.clone();
        Box::pin(async move {
            let _guard = storage::lock(&path).await?;
            let bytes = match storage::read_private(&path, 262_144).await {
                Ok(bytes) => Some(bytes),
                Err(error)
                    if error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            let (policy, bytes) =
                tokio::task::spawn_blocking(move || -> Result<_, StorageError> {
                    let mut record = if let Some(bytes) = bytes {
                        let record: Record = serde_json::from_slice(&bytes)?;
                        if record.version != 1 || record.initialized_providers.len() > 256 {
                            return Err("Invalid or unsupported model allowlist".into());
                        }
                        domain::ModelAllowlist::new(record.patterns.clone())?;
                        record
                    } else {
                        Record {
                            version: 1,
                            ..Record::default()
                        }
                    };
                    match update {
                        ModelAllowlistUpdate::Replace(policy) => {
                            record.patterns = policy
                                .patterns()
                                .iter()
                                .map(|pattern| pattern.as_str().to_owned())
                                .collect();
                            record.customized = true;
                        }
                        ModelAllowlistUpdate::InitializeProvider(provider) => {
                            if !record.customized
                                && !record.initialized_providers.contains(&provider.0)
                            {
                                record.patterns.push(format!("{provider}/*"));
                                record.initialized_providers.push(provider.0);
                            }
                        }
                    }
                    if record.initialized_providers.len() > 256 {
                        return Err("Too many initialized providers".into());
                    }
                    let policy = domain::ModelAllowlist::new(record.patterns.clone())?;
                    let bytes = serde_json::to_vec_pretty(&record)?;
                    if bytes.len() > 262_144 {
                        return Err("The model allowlist exceeds its size limit".into());
                    }
                    Ok((policy, bytes))
                })
                .await??;
            storage::replace(&path, &bytes).await?;
            Ok(policy)
        })
    }
}
