use std::path::Path;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub(super) async fn call(
    repository: &Path,
    arguments: &[&str],
    input: Option<&str>,
    cancellation: &CancellationToken,
) -> Result<String, Error> {
    if cancellation.is_cancelled() {
        return Err("Managed Git stopped before admission".into());
    }
    let mut child = Command::new("git")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
        ])
        .args(arguments)
        .current_dir(repository)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("Git stdin unavailable")?;
    if let Some(input) = input {
        stdin.write_all(input.as_bytes()).await?;
    }
    drop(stdin);
    let stdout = child.stdout.take().ok_or("Git stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("Git stderr unavailable")?;
    let operation = async {
        let mut output = Vec::new();
        let mut error = Vec::new();
        let mut stdout = stdout.take(1_048_577);
        let mut stderr = stderr.take(1_048_577);
        tokio::try_join!(
            stdout.read_to_end(&mut output),
            stderr.read_to_end(&mut error)
        )?;
        if output.len() > 1_048_576 || error.len() > 1_048_576 {
            return Err::<_, Error>(
                "Managed Git output exceeds its bound; inspect without replay".into(),
            );
        }
        let status = child.wait().await?;
        if !status.success() {
            return Err(format!(
                "Managed Git failed ({status}): {}",
                String::from_utf8_lossy(&error)
            )
            .into());
        }
        Ok(String::from_utf8(output)?.trim_end_matches('\n').to_owned())
    };
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err("Managed Git interrupted; reconcile native facts before continuation".into()),
        result = operation => result,
    }
}
