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
pub fn create_process(
    root: &Path,
) -> impl Fn(PathBuf, domain::CreateProcessArguments, CancellationToken) -> Future + Send + Sync + use<>
{
    let root = root.to_owned();
    move |workspace, arguments, cancellation| {
        let root = root.clone();
        Box::pin(async move {
            let _ = &root;
            if cancellation.is_cancelled() {
                return Ok(domain::ToolResult::cancelled());
            }
            if arguments.command.trim().is_empty() {
                return Err("Command must not be empty".into());
            }
            let mut command = tokio::process::Command::new("/bin/bash");
            command
                .arg("--noprofile")
                .arg("--norc")
                .arg("-c")
                .arg(&arguments.command)
                .current_dir(workspace);
            tool_io::process(
                command,
                arguments.timeout_seconds.unwrap_or(120),
                cancellation,
            )
            .await
        })
    }
}
