use crate::repository;
use crate::tool_io;
use std::path::Component;
use std::path::Path;

pub(crate) async fn inspect(
    workspace: &Path,
    value: &str,
) -> Result<Option<String>, tool_io::Error> {
    let (canonical, _) = repository::identity(workspace).await?;
    if canonical != workspace
        || value.is_empty()
        || Path::new(value)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("Setup target must remain in its original repository".into());
    }
    let mut path = workspace.to_owned();
    for component in Path::new(value).components() {
        path.push(component);
        match tokio::fs::symlink_metadata(&path).await {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("Setup does not follow symbolic links".into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(Some(tool_io::text(&path).await?))
}

pub(crate) async fn roots(
    workspace: &Path,
) -> Result<Vec<(String, Option<String>)>, tool_io::Error> {
    let mut roots = Vec::new();
    for name in [
        "AGENTS.md",
        "CLAUDE.md",
        ".harness/instructions/config.json",
    ] {
        roots.push((name.into(), inspect(workspace, name).await?));
    }
    Ok(roots)
}
