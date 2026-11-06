use crate::tool_io;

mod arguments;
pub(crate) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    arguments::decode(json)
}
pub(crate) fn definition() -> serde_json::Value {
    arguments::definition()
}
use crate::tool_io::Future;
use std::path::Path;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
pub fn create_file(
    root: &Path,
) -> impl Fn(PathBuf, domain::CreateFileArguments, CancellationToken) -> Future + Send + Sync + use<>
{
    let root = root.to_owned();
    move |workspace, arguments, cancellation| {
        let root = root.clone();
        Box::pin(async move {
            let _ = &root;
            if cancellation.is_cancelled() {
                return Ok(domain::ToolResult::cancelled());
            }
            let path = tool_io::path(&workspace, &arguments.path);
            let _guard = tool_io::lock(&root, &path).await?;
            let expected = match tokio::fs::symlink_metadata(&path).await {
                Ok(m) if m.is_file() && !m.file_type().is_symlink() => {
                    Some(tool_io::bytes(&path).await?)
                }
                Ok(_) => return Err("Destination must be a regular file or absent".into()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.into()),
            };
            tool_io::replace(&path, arguments.content, expected, cancellation).await
        })
    }
}
