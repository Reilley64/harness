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
pub fn update_file(
    root: &Path,
) -> impl Fn(PathBuf, domain::UpdateFileArguments, CancellationToken) -> Future + Send + Sync + use<>
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
            let original = tool_io::text(&path).await?;
            if arguments.edits.is_empty() || arguments.edits.len() > 32 {
                return Err("Edit requires 1–32 replacements".into());
            }
            let mut ranges = Vec::new();
            for edit in arguments.edits {
                if edit.old_text.is_empty() {
                    return Err("Old text must not be empty".into());
                }
                let start = original
                    .find(&edit.old_text)
                    .ok_or("Old text did not match; read the file before retrying")?;
                let next = start
                    + original[start..]
                        .chars()
                        .next()
                        .ok_or("Empty match")?
                        .len_utf8();
                if original[next..].contains(&edit.old_text) {
                    return Err(
                        "Old text must match exactly once, including overlapping occurrences"
                            .into(),
                    );
                }
                ranges.push((start, start + edit.old_text.len(), edit.new_text));
            }
            ranges.sort_by_key(|r| r.0);
            if ranges.windows(2).any(|r| r[0].1 > r[1].0) {
                return Err("Edit ranges overlap".into());
            }
            let count = ranges.len();
            let mut updated = original.clone();
            for (start, end, replacement) in ranges.into_iter().rev() {
                updated.replace_range(start..end, &replacement);
            }
            let before = original.clone();
            let after = updated.clone();
            let label = path.to_string_lossy().into_owned();
            let mut diff = tokio::task::spawn_blocking(move || {
                similar::TextDiff::configure()
                    .timeout(std::time::Duration::from_secs(1))
                    .diff_lines(&before, &after)
                    .unified_diff()
                    .header(&label, &label)
                    .to_string()
            })
            .await?;
            let truncated = diff.len() > tool_io::CAP;
            if truncated {
                let mut end = tool_io::CAP;
                while !diff.is_char_boundary(end) {
                    end -= 1;
                }
                diff.truncate(end);
                diff.push_str("\n[… diff truncated …]");
            }
            let mut result =
                tool_io::replace(&path, updated, Some(original.into_bytes()), cancellation).await?;
            if result.status == domain::ToolStatus::Succeeded {
                result.output = format!("Changed {count} locations. {}\n{}", result.output, diff);
                result.truncated = truncated;
            }
            Ok(result)
        })
    }
}
