use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: u32,
    model_id: String,
}

pub fn read_decider(
    path: &Path,
) -> impl Fn() -> PortFuture<Result<Option<String>, Error>> + Send + Sync + use<> {
    let path = path.to_owned();
    move || {
        let path = path.clone();
        Box::pin(async move {
            match tokio::fs::symlink_metadata(&path).await {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
                Ok(_) => {}
            }
            let bytes = storage::read_private(&path, 4096).await?;
            let r: Record =
                tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes)).await??;
            if r.format != 1
                || r.model_id.is_empty()
                || r.model_id.len() > 512
                || r.model_id.chars().any(char::is_control)
            {
                return Err("Invalid native Decider preference".into());
            }
            Ok(Some(r.model_id))
        })
    }
}
