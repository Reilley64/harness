mod arguments;
mod record;
pub(crate) use arguments::decode;
pub(crate) use arguments::definition;

use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::instructions;
use application::ports::PortFuture;
use std::path::Path;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn create_research(
    directory: &Path,
) -> impl Fn(
    domain::ChildExecution,
    std::path::PathBuf,
    CancellationToken,
) -> PortFuture<Result<String, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |execution, workspace, cancellation| {
        let directory = directory.clone();
        Box::pin(async move {
            let execution = tokio::task::spawn_blocking(move || {
                execution.validate()?;
                instructions::validate(&execution.instructions)?;
                conversations::validate_id(&execution.agent_id)?;
                Ok::<_, Error>(execution)
            })
            .await??;
            if !workspace.is_absolute() || cancellation.is_cancelled() {
                return Err("Research creation is stopped or has no saved workspace".into());
            }
            let id = execution.agent_id.clone();
            let _guard = conversations::lock(&directory, &id).await?;
            match tokio::fs::symlink_metadata(directory.join(format!("{id}.json"))).await {
                Ok(_) => return Err("Research conversation already exists; inspect the accepted claim without replay".into()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let now = conversations::now()?;
            let bytes = tokio::task::spawn_blocking(move || {
                encode_bounded(&Record::new(execution, workspace, now), 16 * 1024 * 1024)
            })
            .await??;
            if cancellation.is_cancelled() {
                return Err("Research creation stopped before write admission".into());
            }
            conversations::write(&directory, &id, &bytes).await?;
            Ok(id)
        })
    }
}
