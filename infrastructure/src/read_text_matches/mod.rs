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
pub fn read_text_matches(
    root: &Path,
) -> impl Fn(PathBuf, domain::ReadTextMatchesArguments, CancellationToken) -> Future + Send + Sync + use<>
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
                let source = if arguments.literal.unwrap_or(false) {
                    regex::escape(&arguments.pattern)
                } else {
                    arguments.pattern
                };
                let regex = regex::RegexBuilder::new(&source)
                    .case_insensitive(arguments.ignore_case.unwrap_or(false))
                    .size_limit(2 * 1024 * 1024)
                    .build()?;
                let glob = arguments
                    .glob
                    .map(|g| globset::Glob::new(&g).map(|g| g.compile_matcher()))
                    .transpose()?;
                let start = std::time::Instant::now();
                let mut matches = Vec::new();
                let mut visited = 0;
                let mut bytes_used = 0;
                let mut bounded = false;
                let context = arguments.context_lines.unwrap_or(0).min(5);
                for entry in tool_io::walk(&path) {
                    if cancellation.is_cancelled() {
                        return Ok(domain::ToolResult::cancelled());
                    }
                    visited += 1;
                    if visited > 100000 || start.elapsed().as_secs() >= 10 {
                        return Err("Text search exceeded its budget; narrow the path".into());
                    }
                    let entry = entry?;
                    if !entry.file_type().is_some_and(|f| f.is_file()) {
                        continue;
                    }
                    let relative = entry.path().strip_prefix(&path).unwrap_or(entry.path());
                    if glob.as_ref().is_some_and(|g| !g.is_match(relative)) {
                        continue;
                    }
                    if entry.metadata()?.len() > tool_io::CAP as u64 {
                        continue;
                    }
                    let bytes = std::fs::read(entry.path())?;
                    if bytes.contains(&0) {
                        continue;
                    }
                    let Ok(text) = String::from_utf8(bytes) else {
                        continue;
                    };
                    let lines: Vec<_> = text.lines().collect();
                    for (i, line) in lines.iter().enumerate() {
                        if cancellation.is_cancelled() {
                            return Ok(domain::ToolResult::cancelled());
                        }
                        if regex.is_match(line) {
                            let mut excerpt = String::new();
                            for (j, context_line) in lines
                                .iter()
                                .enumerate()
                                .take((i + context + 1).min(lines.len()))
                                .skip(i.saturating_sub(context))
                            {
                                excerpt.push_str(&format!(
                                    "{}:{}: {}\n",
                                    entry.path().display(),
                                    j + 1,
                                    context_line
                                ));
                            }
                            bytes_used += excerpt.len();
                            matches.push(excerpt);
                            if bytes_used >= tool_io::CAP {
                                bounded = true;
                                break;
                            }
                            if bounded || matches.len() > limit {
                                break;
                            }
                        }
                    }
                    if matches.len() > limit {
                        break;
                    }
                }
                let truncated = bounded || matches.len() > limit;
                matches.truncate(limit);
                let mut result = domain::ToolResult::success(matches.join("\n"));
                result.truncated = truncated;
                Ok(result)
            })
            .await?
        })
    }
}
