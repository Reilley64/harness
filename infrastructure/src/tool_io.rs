use crate::storage;

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use application::error_context::ResultExt;
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Future = application::ports::PortFuture<Result<domain::ToolResult, Error>>;
pub(crate) const CAP: usize = 1_048_576;

pub(crate) fn path(workspace: &Path, value: &str) -> PathBuf {
    workspace.join(value)
}
pub(crate) fn limit(value: Option<usize>) -> usize {
    value.unwrap_or(100).clamp(1, 100)
}
pub(crate) async fn bytes(path: &Path) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    let mut options = tokio::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK);
    let file = options
        .open(path)
        .await
        .context("Could not open tool input file")?;
    if !file.metadata().await?.is_file() {
        return Err("Tool input must be a regular file".into());
    }
    file.take((CAP + 1) as u64).read_to_end(&mut bytes).await?;
    if bytes.len() > CAP {
        return Err("File exceeds the 1 MiB tool input limit; narrow the operation".into());
    }
    Ok(bytes)
}
pub(crate) async fn text(path: &Path) -> Result<String, Error> {
    Ok(String::from_utf8(bytes(path).await?).context("The tool accepts UTF-8 text files only")?)
}

pub(crate) async fn lock(root: &Path, path: &Path) -> Result<std::fs::File, Error> {
    use std::hash::Hash;
    use std::hash::Hasher;
    let canonical = if tokio::fs::try_exists(path).await? {
        tokio::fs::canonicalize(path).await?
    } else {
        let parent = path.parent().ok_or("File needs a parent")?;
        tokio::fs::create_dir_all(parent).await?;
        tokio::fs::canonicalize(parent)
            .await?
            .join(path.file_name().ok_or("Missing file name")?)
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    canonical.hash(&mut hash);
    let directory = root.join("tool-locks");
    storage::private_directory(&directory).await?;
    let mut options = tokio::fs::OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options
        .open(directory.join(format!("{:016x}.lock", hash.finish())))
        .await?
        .into_std()
        .await;
    tokio::task::spawn_blocking(move || {
        file.try_lock()
            .context("Another Harness tool is modifying this file")?;
        Ok(file)
    })
    .await?
}
pub(crate) async fn replace(
    path: &Path,
    content: String,
    expected: Option<Vec<u8>>,
    cancellation: CancellationToken,
) -> Result<domain::ToolResult, Error> {
    if content.len() > CAP {
        return Err("File contents exceed 1 MiB".into());
    }
    if cancellation.is_cancelled() {
        return Ok(domain::ToolResult::cancelled());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        use std::io::Write;
        if cancellation.is_cancelled() {
            return Ok(domain::ToolResult::cancelled());
        }
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(m) => Some(m),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if metadata
            .as_ref()
            .is_some_and(|m| !m.is_file() || m.file_type().is_symlink())
        {
            return Err("Refusing to replace a nonregular or symlink destination".into());
        }
        let current = match std::fs::read(&path) {
            Ok(v) => Some(v),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if current != expected {
            return Err("File changed during the operation; read it again before retrying".into());
        }
        let parent = path.parent().ok_or("File requires a parent directory")?;
        let mut builder = tempfile::Builder::new();
        #[cfg(unix)]
        if metadata.is_none() {
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(std::fs::Permissions::from_mode(0o666));
        }
        let mut temporary = builder.tempfile_in(parent)?;
        if let Some(m) = metadata {
            temporary.as_file().set_permissions(m.permissions())?;
        }
        temporary.write_all(content.as_bytes())?;
        temporary.flush()?;
        temporary.as_file().sync_all()?;
        if cancellation.is_cancelled() {
            return Ok(domain::ToolResult::cancelled());
        }
        let latest_metadata = std::fs::symlink_metadata(&path);
        if latest_metadata
            .as_ref()
            .is_ok_and(|m| !m.is_file() || m.file_type().is_symlink())
        {
            return Err("Destination changed before commit".into());
        }
        let latest = match std::fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        if latest != expected {
            return Err("File changed before commit; read it again before retrying".into());
        }
        if expected.is_none() {
            temporary.persist_noclobber(&path)?;
        } else {
            temporary.persist(&path)?;
        }
        if let Err(error) = std::fs::File::open(parent).and_then(|directory| directory.sync_all()) {
            return Ok(uncertain(&error));
        }
        Ok(domain::ToolResult::success(format!(
            "Wrote {} bytes to {}",
            content.len(),
            path.display()
        )))
    })
    .await?
}

pub(crate) fn process_environment(command: &mut tokio::process::Command) {
    command
        .env_clear()
        .envs(std::env::vars_os().filter(|(name, _)| {
            name.to_str().is_some_and(|name| {
                matches!(
                    name,
                    "PATH"
                        | "HOME"
                        | "LANG"
                        | "LANGUAGE"
                        | "LC_ALL"
                        | "LC_CTYPE"
                        | "LC_COLLATE"
                        | "LC_MONETARY"
                        | "LC_NUMERIC"
                        | "LC_TIME"
                        | "LC_MESSAGES"
                        | "LC_PAPER"
                        | "LC_NAME"
                        | "LC_ADDRESS"
                        | "LC_TELEPHONE"
                        | "LC_MEASUREMENT"
                        | "LC_IDENTIFICATION"
                        | "TZ"
                        | "TMPDIR"
                        | "TMP"
                        | "TEMP"
                        | "TERM"
                        | "COLORTERM"
                        | "NO_COLOR"
                        | "FORCE_COLOR"
                        | "SYSTEMROOT"
                        | "WINDIR"
                        | "PATHEXT"
                        | "COMSPEC"
                        | "RUSTUP_HOME"
                        | "RUSTUP_TOOLCHAIN"
                        | "CARGO_HOME"
                        | "XDG_CONFIG_HOME"
                        | "XDG_CACHE_HOME"
                        | "XDG_DATA_HOME"
                        | "MISE_DATA_DIR"
                        | "MISE_CONFIG_DIR"
                )
            })
        }));
}

pub(crate) async fn process(
    mut command: tokio::process::Command,
    seconds: u64,
    cancellation: CancellationToken,
) -> Result<domain::ToolResult, Error> {
    use std::process::Stdio;
    process_environment(&mut command);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn().context("Could not start tool executable")?;
    let pid = child.id();
    let _group = ProcessGroup(pid);
    let stdout = child.stdout.take().ok_or("Missing tool stdout")?;
    let stderr = child.stderr.take().ok_or("Missing tool stderr")?;
    async fn drain(
        mut input: impl tokio::io::AsyncRead + Unpin,
    ) -> Result<(Vec<u8>, bool), std::io::Error> {
        let mut out = Vec::new();
        let mut chunk = [0u8; 8192];
        let mut truncated = false;
        loop {
            let n = input.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            let keep = n.min(CAP.saturating_sub(out.len()));
            out.extend_from_slice(&chunk[..keep]);
            truncated |= keep < n;
        }
        Ok((out, truncated))
    }
    let mut out = tokio::spawn(drain(stdout));
    let mut err = tokio::spawn(drain(stderr));
    let timeout = tokio::time::sleep(Duration::from_secs(seconds.clamp(1, 600)));
    tokio::pin!(timeout);
    let mut stopped = false;
    let mut timed_out = false;
    let status = tokio::select! {
     result=child.wait()=>result.map_err(Error::from),
     _=cancellation.cancelled()=>{stopped=true;stop(&mut child,pid).await},
     _=&mut timeout=>{timed_out=true;stop(&mut child,pid).await},
    };
    #[cfg(unix)]
    if let Some(pid) = pid {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
    let status = match status {
        Ok(status) => status,
        Err(error) => {
            out.abort();
            err.abort();
            return Ok(uncertain(error.as_ref()));
        }
    };
    let outputs = tokio::time::timeout(Duration::from_secs(3), async {
        Ok::<_, Error>(((&mut out).await??, (&mut err).await??))
    })
    .await;
    let ((stdout, truncated_out), (stderr, truncated_err)) = match outputs {
        Ok(Ok(outputs)) => outputs,
        Ok(Err(error)) => {
            out.abort();
            err.abort();
            return Ok(uncertain(error.as_ref()));
        }
        Err(error) => {
            out.abort();
            err.abort();
            return Ok(uncertain(&error));
        }
    };
    let mut result = domain::ToolResult::success(format!(
        "Exit: {}\n{}{}",
        status,
        String::from_utf8_lossy(&stdout),
        if stderr.is_empty() {
            String::new()
        } else {
            format!("\n[stderr]\n{}", String::from_utf8_lossy(&stderr))
        }
    ));
    result.truncated = truncated_out || truncated_err;
    if stopped {
        result.status = domain::ToolStatus::Cancelled;
    } else if timed_out {
        result.status = domain::ToolStatus::Failed;
        result.output = format!(
            "Timed out after {seconds}s; process group stopped.\n{}",
            result.output
        );
    } else if !status.success() {
        result.status = domain::ToolStatus::Failed;
    }
    Ok(result)
}
struct ProcessGroup(Option<u32>);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.0 {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
}
fn uncertain(error: &(dyn std::error::Error + 'static)) -> domain::ToolResult {
    domain::ToolResult {
        status: domain::ToolStatus::Unknown,
        output: format!(
            "Outcome or cleanup could not be confirmed; inspect effects before retrying: {}",
            application::error_context::diagnostic(error)
        ),
        artifact: None,
        truncated: false,
    }
}
async fn stop(
    child: &mut tokio::process::Child,
    pid: Option<u32>,
) -> Result<std::process::ExitStatus, Error> {
    #[cfg(unix)]
    if let Some(pid) = pid {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGTERM);
        }
    }
    match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
        Ok(status) => Ok(status?),
        Err(_) => {
            #[cfg(unix)]
            if let Some(pid) = pid {
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
            }
            child.start_kill()?;
            Ok(child.wait().await?)
        }
    }
}

pub(crate) fn walk(path: &Path) -> ignore::Walk {
    ignore::WalkBuilder::new(path)
        .hidden(false)
        .require_git(false)
        .follow_links(false)
        .build()
}
