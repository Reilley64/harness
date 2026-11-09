mod record;

use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[ohno::error]
#[display("Execution diagnostic persistence failed with {} retained originals", diagnostics.len())]
struct Retained {
    diagnostics: Vec<Error>,
}

pub fn create_execution_diagnostics(
    directory: &Path,
) -> impl Fn(String, String, Vec<Error>) -> PortFuture<Result<(), Error>> + Send + Sync + use<> {
    let directory = directory.to_owned();
    move |parent_id, execution_id, diagnostics| {
        let directory = directory.clone();
        Box::pin(async move {
            let path = directory.join(format!("{execution_id}.json"));
            let (diagnostics, encoded) = tokio::task::spawn_blocking(move || {
                let encoded = record::encode(parent_id, execution_id, &diagnostics);
                (diagnostics, encoded)
            })
            .await?;
            let result = async {
                let bytes = encoded?;
                storage::private_directory(&directory).await?;
                let _guard = storage::lock(&path).await?;
                match tokio::fs::symlink_metadata(&path).await {
                    Ok(_) => {
                        if storage::read_private(&path, 4 * 1024 * 1024).await? != bytes {
                            return Err::<(), Error>("An execution diagnostic record already retains different original facts".into());
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => storage::replace(&path, &bytes).await?,
                    Err(error) => return Err(error.into()),
                }
                Ok(())
            }.await;
            result.map_err(|error| Box::new(Retained::caused_by(diagnostics, error)) as Error)
        })
    }
}
