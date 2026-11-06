use crate::storage;
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
const LANGUAGES: &[&str] = &[
    "rust",
    "typescript",
    "tsx",
    "javascript",
    "jsx",
    "python",
    "go",
    "java",
    "c",
    "cpp",
    "csharp",
    "ruby",
    "bash",
    "json",
    "yaml",
    "html",
    "css",
];

pub fn read_code_matches(
    root: &Path,
) -> impl Fn(PathBuf, domain::ReadCodeMatchesArguments, CancellationToken) -> Future + Send + Sync + use<>
{
    let root = root.to_owned();
    move |workspace, arguments, cancellation| {
        let root = root.clone();
        Box::pin(async move {
            let _ = &root;
            if cancellation.is_cancelled() {
                return Ok(domain::ToolResult::cancelled());
            }
            let language = arguments.language.trim().to_ascii_lowercase();
            if !LANGUAGES.contains(&language.as_str()) {
                return Err(format!(
                    "Unsupported ast-grep language; supported built-ins: {}",
                    LANGUAGES.join(", ")
                )
                .into());
            }
            let config = root.join("tools/ast-grep/sgconfig.yml");
            storage::private_directory(
                config
                    .parent()
                    .ok_or("Missing ast-grep configuration directory")?,
            )
            .await?;
            storage::replace(&config, b"ruleDirs: []\n").await?;
            let mut command = tokio::process::Command::new("ast-grep");
            command
                .args([
                    "run",
                    "--json=stream",
                    "--color=never",
                    "--custom-languages",
                    "ignore",
                    "--config",
                ])
                .arg(config)
                .arg("--lang")
                .arg(language)
                .arg("--pattern")
                .arg(arguments.pattern);
            if let Some(glob) = arguments.glob {
                command.arg("--globs").arg(glob);
            }
            command
                .arg("--")
                .arg(tool_io::path(
                    &workspace,
                    arguments.path.as_deref().unwrap_or("."),
                ))
                .current_dir(workspace);
            let mut result = tool_io::process(command, 10, cancellation).await?;
            if result.status == domain::ToolStatus::Cancelled {
                return Ok(result);
            }
            if result.output.starts_with("Exit: exit status: 1\n") {
                return Ok(domain::ToolResult::success("No structural matches".into()));
            }
            if result.status != domain::ToolStatus::Succeeded {
                return Ok(result);
            }
            let limit = tool_io::limit(arguments.limit);
            let mut matches = Vec::new();
            for line in result.output.lines().skip(1).take(limit + 1) {
                let value: serde_json::Value = match serde_json::from_str(line) {
                    Ok(value) => value,
                    Err(_) if result.truncated => break,
                    Err(error) => return Err(error.into()),
                };
                matches.push(format!(
                    "{}:{}:{}: {}",
                    value["file"].as_str().unwrap_or(""),
                    value["range"]["start"]["line"].as_u64().unwrap_or(0) + 1,
                    value["range"]["start"]["column"].as_u64().unwrap_or(0) + 1,
                    value["text"].as_str().unwrap_or("")
                ));
            }
            result.truncated |= matches.len() > limit;
            matches.truncate(limit);
            result.output = matches.join("\n");
            Ok(result)
        })
    }
}
