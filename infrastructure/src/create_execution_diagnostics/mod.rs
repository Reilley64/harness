mod record;

use crate::storage;
use application::ports::PortFuture;
use sha2::Digest;
use sha2::Sha256;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[ohno::error]
#[display("Execution diagnostic persistence failed with {} retained originals", diagnostics.len())]
struct Retained {
    diagnostics: Vec<Error>,
}

pub fn create_execution_diagnostics(
    directory: &Path,
) -> impl Fn(String, String, Option<String>, Vec<Error>) -> PortFuture<Result<(), Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |parent_id, execution_id, run_id, diagnostics| {
        let directory = directory.clone();
        Box::pin(async move {
            let scoped = run_id.is_some();
            let directory = match &run_id {
                Some(run) => directory.join(&execution_id).join(run),
                None => directory,
            };
            let execution = execution_id.clone();
            let (diagnostics, encoded) = tokio::task::spawn_blocking(move || {
                let encoded = record::encode(parent_id, execution_id, run_id, &diagnostics);
                (diagnostics, encoded)
            })
            .await?;
            let result = async {
                let bytes = encoded?;
                let path = directory.join(if scoped {
                    format!("{:x}.json", Sha256::digest(&bytes))
                } else {
                    format!("{execution}.json")
                });
                storage::private_directory(&directory).await?;
                let guard_path = if scoped { directory.join("batches.json") } else { path.clone() };
                let _guard = storage::lock(&guard_path).await?;
                match tokio::fs::symlink_metadata(&path).await {
                    Ok(_) => {
                        if storage::read_private(&path, 4 * 1024 * 1024).await? != bytes {
                            return Err::<(), Error>("An execution diagnostic record already retains different original facts".into());
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        if scoped {
                            let mut entries = tokio::fs::read_dir(&directory).await?;
                            let mut count = 0;
                            while entries.next_entry().await?.is_some() {
                                count += 1;
                                if count >= 512 { return Err("Run diagnostic storage exceeds its bounded capacity; originals remain in the runtime error".into()) }
                            }
                        }
                        storage::replace(&path, &bytes).await?;
                    }
                    Err(error) => return Err(error.into()),
                }
                Ok(())
            }.await;
            result.map_err(|error| Box::new(Retained::caused_by(diagnostics, error)) as Error)
        })
    }
}
