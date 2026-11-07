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
    let root = matches!(arguments.path.as_str(), "AGENTS.md" | "CLAUDE.md");
    let selection = arguments.path == ".harness/instructions/config.json";
    if root {
        if before.is_none() && roots.iter().take(2).any(|(_, body)| body.is_some()) {
            return Err("Modify the existing root instead of creating a duplicate".into());
        }
        if before
            .as_ref()
            .is_some_and(|base| !arguments.content.starts_with(base))
        {
            return Err("Setup root drafts must preserve all existing content and append only missing navigation".into());
        }
    } else if selection {
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
    } else if !matches!(
        arguments.path.as_str(),
        "docs/agents/issue-tracker.md"
            | "docs/agents/domain.md"
            | "docs/agents/triage-labels.md"
            | "docs/agents/wiki.md"
    ) {
        let wiki = setup_files::inspect(&workspace, "docs/agents/wiki.md").await?.or(proposed_wiki).ok_or("Propose wiki guidance with its exact index pointer before proposing a new wiki index")?;
        if !arguments.path.ends_with(".md")
            || before.is_some()
            || (!wiki.contains(&format!("`{}`", arguments.path))
                && !wiki.contains(&format!("]({})", arguments.path)))
        {
            return Err(
                "Only a missing index explicitly referenced by wiki guidance is a Setup output"
                    .into(),
            );
        }
    } else if before.is_some() {
        return Err("Setup only fills missing Markdown guidance/wiki outputs; existing documents are preserved".into());
    }
    let draft = domain::SetupDraft {
        workspace,
        path: arguments.path,
        before,
        after: arguments.content,
        roots,
    };
    draft.validate()?;
    Ok(draft)
}
