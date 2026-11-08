use crate::setup_files;
use crate::tool_io;
use application::ports::PortFuture;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub fn update_setup_draft(
    root: &Path,
) -> impl Fn(
    domain::SetupDraft,
    CancellationToken,
) -> PortFuture<Result<domain::ToolResult, tool_io::Error>>
+ Send
+ Sync
+ use<> {
    let root = root.to_owned();
    move |draft, cancellation| {
        let root = root.clone();
        Box::pin(async move {
            draft.validate()?;
            if cancellation.is_cancelled() {
                return Ok(domain::ToolResult::cancelled());
            }
            if setup_files::roots(&draft.workspace).await? != draft.roots {
                return Ok(domain::ToolResult::failed(
                    "Root guidance changed; inspect and generate a fresh proposal".into(),
                ));
            }
            if setup_files::inspect(&draft.workspace, &draft.path).await? != draft.before {
                return Ok(domain::ToolResult::failed(
                    "Target changed; inspect and generate a fresh proposal".into(),
                ));
            }
            let path = draft.workspace.join(&draft.path);
            let _guard = tool_io::lock(&root, &path).await?;
            if setup_files::roots(&draft.workspace).await? != draft.roots
                || setup_files::inspect(&draft.workspace, &draft.path).await? != draft.before
            {
                return Ok(domain::ToolResult::failed(
                    "Setup proposal is stale; no file was written".into(),
                ));
            }
            tool_io::replace(
                &path,
                draft.after,
                draft.before.map(String::into_bytes),
                cancellation,
            )
            .await
        })
    }
}
