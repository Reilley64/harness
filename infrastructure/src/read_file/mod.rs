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
pub fn read_file(
    root: &Path,
) -> impl Fn(PathBuf, domain::ReadFileArguments, CancellationToken) -> Future + Send + Sync + use<>
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
            let text = tool_io::text(&path).await?;
            let offset = arguments.offset.unwrap_or(1);
            let limit = arguments.limit.unwrap_or(200).clamp(1, 2000);
            if offset == 0 {
                return Err("Line offset starts at 1".into());
            }
            let lines: Vec<_> = text.lines().collect();
            let output = lines
                .iter()
                .enumerate()
                .skip(offset - 1)
                .take(limit)
                .map(|(i, line)| format!("{}: {}", i + 1, line))
                .collect::<Vec<_>>()
                .join("\n");
            let mut result = domain::ToolResult::success(format!(
                "{} · lines {}–{} of {}\n{}",
                path.display(),
                offset,
                (offset - 1 + limit).min(lines.len()),
                lines.len(),
                output
            ));
            result.truncated = offset - 1 + limit < lines.len();
            Ok(result)
        })
    }
}
