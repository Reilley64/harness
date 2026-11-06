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
pub fn read_paths(
    root: &Path,
) -> impl Fn(PathBuf, domain::ReadPathsArguments, CancellationToken) -> Future + Send + Sync + use<>
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
            tokio::task::spawn_blocking(move || {
                let pattern = globset::Glob::new(&arguments.pattern)?.compile_matcher();
                let start = std::time::Instant::now();
                let mut matches = Vec::new();
                let mut visited = 0;
                for entry in tool_io::walk(&path) {
                    if cancellation.is_cancelled() {
                        return Ok(domain::ToolResult::cancelled());
                    }
                    visited += 1;
                    if visited > 100000 || start.elapsed().as_secs() >= 10 {
                        return Err("Path search exceeded its budget; narrow the path".into());
                    }
                    let entry = entry?;
                    let relative = entry.path().strip_prefix(&path).unwrap_or(entry.path());
                    if !relative.as_os_str().is_empty() && pattern.is_match(relative) {
                        matches.push(relative.to_string_lossy().into_owned());
                        if matches.len() > limit {
                            break;
                        }
                    }
                }
                matches.sort();
                let truncated = matches.len() > limit;
                matches.truncate(limit);
                let mut result = domain::ToolResult::success(matches.join("\n"));
                result.truncated = truncated;
                Ok(result)
            })
            .await?
        })
    }
}
