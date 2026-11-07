use crate::conversations::StorageError;
use std::path::Path;

pub(crate) async fn identity(
    workspace: &Path,
) -> Result<(std::path::PathBuf, String), StorageError> {
    let workspace = tokio::fs::canonicalize(workspace).await?;
    if !tokio::fs::metadata(&workspace).await?.is_dir() {
        return Err("Workspace must be a directory".into());
    }
    let result = tokio::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&workspace)
        .kill_on_drop(true)
        .output();
    let workspace = match tokio::time::timeout(std::time::Duration::from_secs(5), result).await {
        Ok(Ok(output)) if output.status.success() && output.stdout.len() <= 16_384 => {
            let path = String::from_utf8(output.stdout)?;
            let path = path
                .strip_suffix('\n')
                .ok_or("Invalid Git workspace response")?;
            tokio::fs::canonicalize(path).await?
        }
        _ => workspace,
    };
    let id = uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        workspace.as_os_str().as_encoded_bytes(),
    )
    .to_string();
    Ok((workspace, id))
}
