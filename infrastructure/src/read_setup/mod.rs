use crate::repository;
use crate::tool_io;
use application::ports::PortFuture;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub fn read_setup()
-> impl Fn(PathBuf, CancellationToken) -> PortFuture<Result<domain::SetupStatus, tool_io::Error>>
+ Send
+ Sync {
    move |workspace, cancellation| {
        Box::pin(async move {
            tokio::select! {
                biased;
                _ = cancellation.cancelled() => Err("Setup inspection cancelled".into()),
                result = inspect(workspace) => result,
            }
        })
    }
}

async fn inspect(workspace: PathBuf) -> Result<domain::SetupStatus, tool_io::Error> {
    let (workspace, _) = repository::identity(&workspace).await?;
    let mut roots = Vec::new();
    let mut triage = false;
    for root in ["AGENTS.md", "CLAUDE.md"] {
        if present(&workspace.join(root)).await? {
            roots.push(root.into());
            if let Ok(body) = tool_io::text(&workspace.join(root)).await {
                triage |= body.contains("docs/agents/triage-labels.md");
            }
        }
    }
    let mut missing = Vec::new();
    if roots.is_empty() {
        missing.push("AGENTS.md or CLAUDE.md (ask which to create)".into());
    }
    for path in [
        "docs/agents/issue-tracker.md",
        "docs/agents/domain.md",
        "docs/agents/wiki.md",
    ] {
        if !present(&workspace.join(path)).await? {
            missing.push(path.into());
        }
    }
    if triage && !present(&workspace.join("docs/agents/triage-labels.md")).await? {
        missing.push("docs/agents/triage-labels.md".into());
    }
    Ok(domain::SetupStatus {
        workspace,
        roots,
        missing,
        triage,
    })
}

async fn present(path: &std::path::Path) -> Result<bool, tool_io::Error> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
