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
pub fn read_directory(
    root: &Path,
) -> impl Fn(PathBuf, domain::ReadDirectoryArguments, CancellationToken) -> Future + Send + Sync + use<>
{
    let root = root.to_owned();
    move |workspace, arguments, cancellation| {
        let root = root.clone();
        Box::pin(async move {
            let _ = &root;
            if cancellation.is_cancelled() {
                return Ok(domain::ToolResult::cancelled());
            }
            let path = tool_io::path(&workspace, arguments.path.as_deref().unwrap_or("."));
            let limit = tool_io::limit(arguments.limit);
            let mut directory = tokio::fs::read_dir(path).await?;
            let mut entries = Vec::new();
            while let Some(entry) = directory.next_entry().await? {
                if cancellation.is_cancelled() {
                    return Ok(domain::ToolResult::cancelled());
                }
                if entries.len() >= 100000 {
                    return Err("Directory is too large; narrow the path".into());
                }
                let kind = entry.file_type().await?;
                entries.push(format!(
                    "{}{}",
                    entry.file_name().to_string_lossy(),
                    if kind.is_dir() {
                        "/"
                    } else if kind.is_symlink() {
                        " -> [symlink]"
                    } else {
                        ""
                    }
                ));
            }
            entries.sort();
            let truncated = entries.len() > limit;
            entries.truncate(limit);
            let mut result = domain::ToolResult::success(entries.join("\n"));
            result.truncated = truncated;
            Ok(result)
        })
    }
}
