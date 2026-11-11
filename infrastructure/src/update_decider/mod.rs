use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: u32,
    model_id: String,
}

pub fn update_decider(
    path: &Path,
) -> impl Fn(String) -> PortFuture<Result<String, Error>> + Send + Sync + use<> {
    let path = path.to_owned();
    move |model_id| {
        let path = path.clone();
        Box::pin(async move {
            if model_id.is_empty() || model_id.len() > 512 || model_id.chars().any(char::is_control)
            {
                return Err("Invalid native Decider identity".into());
            }
            let _guard = storage::lock(&path).await?;
            if tokio::fs::symlink_metadata(&path).await.is_ok() {
                let bytes = storage::read_private(&path, 4096).await?;
                let prior: Record =
                    tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes)).await??;
                if prior.format != 1 {
                    return Err("Unsupported native Decider preference".into());
                }
            }
            let encoded = serde_json::to_vec(&Record {
                format: 1,
                model_id: model_id.clone(),
            })?;
            storage::replace(&path, &encoded).await?;
            Ok(model_id)
        })
    }
}
