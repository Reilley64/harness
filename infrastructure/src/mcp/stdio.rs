use crate::tool_io::Error;
use application::error_context::ResultExt;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

const LIMIT: usize = 1_048_576;

pub(super) struct Transport {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    call_sent: bool,
}

impl Transport {
    pub(super) async fn start(
        workspace: &Path,
        command: &str,
        args: &[String],
        environment: &std::collections::BTreeMap<String, String>,
        cancellation: CancellationToken,
    ) -> Result<(Self, Vec<String>), Error> {
        let mut process = Command::new(command);
        process
            .args(args)
            .current_dir(workspace)
            .env_clear()
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(path) = std::env::var_os("PATH") {
            process.env("PATH", path);
        }
        #[cfg(unix)]
        process.process_group(0);

        let mut redactions = Vec::new();
        for (name, reference) in environment {
            if !reference.starts_with("HARNESS_MCP_") {
                return Err(
                    "MCP environment references must use the HARNESS_MCP_ namespace".into(),
                );
            }
            let value = std::env::var_os(reference)
                .ok_or("Required MCP environment reference is not provisioned")?;
            let secret = value
                .to_str()
                .ok_or("MCP provisioned values must be UTF-8")?;
            if secret.is_empty() || secret.len() > 8192 {
                return Err("MCP provisioned value is empty or too long".into());
            }
            redactions.push(secret.to_owned());
            process.env(name, value);
        }
        if cancellation.is_cancelled() {
            return Err("MCP connection cancelled before launch".into());
        }

        let mut child = process.spawn()?;
        let input = child.stdin.take().ok_or("Missing MCP stdin")?;
        let output = BufReader::new(child.stdout.take().ok_or("Missing MCP stdout")?);

        Ok((
            Self {
                child,
                input,
                output,
                call_sent: false,
            },
            redactions,
        ))
    }

    pub(super) fn call_sent(&self) -> bool {
        self.call_sent
    }

    pub(super) fn reset_call_sent(&mut self) {
        self.call_sent = false;
    }

    pub(super) async fn send(&mut self, mut bytes: Vec<u8>, tool_call: bool) -> Result<(), Error> {
        bytes.push(b'\n');
        self.call_sent |= tool_call;
        self.input.write_all(&bytes).await?;
        self.input.flush().await?;
        Ok(())
    }

    pub(super) async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        (&mut self.output)
            .take(LIMIT as u64 + 1)
            .read_until(b'\n', &mut bytes)
            .await?;
        if bytes.is_empty() || bytes.len() > LIMIT || bytes.last() != Some(&b'\n') {
            return Err("MCP connection closed or response exceeded 1 MiB".into());
        }
        Ok(bytes)
    }

    pub(super) async fn close(&mut self) -> Result<(), Error> {
        self.kill_group();
        tokio::time::timeout(Duration::from_secs(2), self.child.kill())
            .await
            .context("MCP process cleanup timed out")?
            .context("MCP process cleanup failed")?;
        Ok(())
    }

    fn kill_group(&self) {
        #[cfg(unix)]
        if let Some(id) = self.child.id().and_then(|id| i32::try_from(id).ok()) {
            unsafe {
                libc::kill(-id, libc::SIGKILL);
            }
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        self.kill_group();
        let _ = self.child.start_kill();
    }
}
