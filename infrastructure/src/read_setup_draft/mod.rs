use crate::repository;
use crate::setup_files;
use crate::tool_io;
use application::error_context::ResultExt;
use application::ports::PortFuture;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub fn read_setup_draft() -> impl Fn(
    PathBuf,
    domain::CreateFileArguments,
    Option<String>,
    CancellationToken,
) -> PortFuture<Result<domain::SetupDraft, tool_io::Error>>
+ Send
+ Sync {
    move |workspace, arguments, proposed_wiki, cancellation| {
        Box::pin(async move {
            tokio::select! {
                biased;
                _ = cancellation.cancelled() => Err("Setup draft inspection cancelled".into()),
                result = prepare(workspace, arguments, proposed_wiki) => result.context("Setup draft rejected; preserve existing files, select the intended root and propose only missing guidance or an explicitly referenced wiki index").map_err(Into::into),
            }
        })
    }
}

async fn prepare(
    workspace: PathBuf,
    arguments: domain::CreateFileArguments,
    proposed_wiki: Option<String>,
) -> Result<domain::SetupDraft, tool_io::Error> {
    let (workspace, _) = repository::identity(&workspace).await?;
    let before = setup_files::inspect(&workspace, &arguments.path).await?;
    let roots = setup_files::roots(&workspace).await?;

    if arguments.path == ".harness/instructions/config.json" {
        let old = before.clone().unwrap_or_else(|| r#"{"format":1}"#.into());
        let content = arguments.content.clone();
        tokio::task::spawn_blocking(move || -> Result<(), tool_io::Error> {
            let old: serde_json::Value = serde_json::from_str(&old)?;
            let mut new: serde_json::Value = serde_json::from_str(&content)?;
            let chosen = new
                .get("root")
                .and_then(|v| v.as_str())
                .ok_or("Select the intended root")?;
            if !matches!(chosen, "AGENTS.md" | "CLAUDE.md") {
                return Err("Setup can select only root AGENTS.md or CLAUDE.md".into());
            }
            let mut old = old;
            old.as_object_mut()
                .ok_or("Invalid instruction configuration")?
                .remove("root");
            new.as_object_mut()
                .ok_or("Invalid instruction configuration")?
                .remove("root");
            if old != new {
                return Err(
                    "Setup root selection must preserve instruction overrides and format".into(),
                );
            }
            Ok(())
        })
        .await??;
    }

    let wiki = setup_files::inspect(&workspace, "docs/agents/wiki.md")
        .await?
        .or(proposed_wiki);
    let draft = domain::SetupDraft {
        workspace,
        path: arguments.path,
        before,
        after: arguments.content,
        roots,
    };
    draft.validate_output(wiki.as_deref())?;
    Ok(draft)
}
