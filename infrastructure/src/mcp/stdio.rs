mod projection;

use crate::tool_io::Error;
use application::error_context::ResultExt;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
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

#[derive(Debug)]
struct RequestRejected;

impl std::fmt::Display for RequestRejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .write_str("MCP server rejected the request; verify access and operation availability")
    }
}

impl std::error::Error for RequestRejected {}

pub(super) struct Client {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next: u64,
    redactions: Arc<[String]>,
}

impl Client {
    pub(super) async fn start(
        workspace: &Path,
        definition: &domain::McpDefinition,
        cancellation: CancellationToken,
    ) -> Result<Self, Error> {
        let domain::McpTransport::Stdio { command, args } = &definition.transport else {
            return Err("Streamable HTTP execution is unavailable; definition saved only".into());
        };

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
        for (name, reference) in &definition.environment {
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
        redactions
            .sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
        redactions.dedup();

        let mut client = Self {
            child,
            input,
            output,
            next: 0,
            redactions: redactions.into(),
        };

        let response = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(30), client.rpc("initialize", json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"harness","version":env!("CARGO_PKG_VERSION")}}))) => result??,
            _ = cancellation.cancelled() => return Err("MCP connection cancelled".into()),
        };
        if !matches!(
            response["protocolVersion"].as_str(),
            Some("2025-06-18" | "2025-03-26" | "2024-11-05")
        ) || !response["capabilities"]["tools"].is_object()
        {
            return Err("Unsupported MCP protocol or missing tools capability".into());
        }

        client
            .send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .await?;

        Ok(client)
    }

    async fn send(&mut self, value: Value) -> Result<(), Error> {
        let mut bytes = serde_json::to_vec(&value)?;
        if bytes.len() > LIMIT {
            return Err("MCP request exceeds 1 MiB".into());
        }

        bytes.push(b'\n');
        self.input.write_all(&bytes).await?;
        self.input.flush().await?;
        Ok(())
    }

    async fn rpc(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.next += 1;
        let id = self.next;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await?;

        for _ in 0..256 {
            let mut bytes = Vec::new();
            (&mut self.output)
                .take(LIMIT as u64 + 1)
                .read_until(b'\n', &mut bytes)
                .await?;
            if bytes.is_empty() || bytes.len() > LIMIT || bytes.last() != Some(&b'\n') {
                return Err("MCP connection closed or response exceeded 1 MiB".into());
            }
            let (mut response, request_id) =
                tokio::task::spawn_blocking(move || -> Result<_, Error> {
                    let response: Value =
                        serde_json::from_slice(&bytes).context("Invalid MCP JSON-RPC response")?;
                    let request_id = response
                        .get("method")
                        .and_then(|_| response.get("id"))
                        .cloned();
                    Ok((response, request_id))
                })
                .await??;

            if response["jsonrpc"] != "2.0" {
                return Err("Invalid MCP JSON-RPC version".into());
            }
            if response.get("method").is_some() {
                if let Some(request_id) = request_id {
                    self.send(json!({"jsonrpc":"2.0","id":request_id,"error":{"code":-32601,"message":"Client capability not supported"}})).await?;
                }
                continue;
            }

            if response["id"] != id {
                return Err("MCP response identity mismatch".into());
            }

            let error = response.get("error");
            let result = response.get("result");
            if error.is_some() == result.is_some() {
                return Err("MCP response must contain exactly one result or error".into());
            }
            if let Some(error) = error {
                if error["code"].as_i64().is_none() || !error["message"].is_string() {
                    return Err("Malformed MCP error response".into());
                }
                return Err(RequestRejected.into());
            }

            return response
                .get_mut("result")
                .map(Value::take)
                .ok_or_else(|| "Missing MCP result".into());
        }

        Err("MCP notification limit exceeded".into())
    }

    async fn list(&mut self) -> Result<Vec<domain::McpTool>, Error> {
        let mut tools = Vec::new();
        let mut names = BTreeSet::new();
        let mut cursor = None;
        let mut size = 0;
        for _ in 0..16 {
            let response = self
                .rpc(
                    "tools/list",
                    cursor
                        .as_ref()
                        .map_or(json!({}), |cursor| json!({"cursor":cursor})),
                )
                .await?;

            let entries = response["tools"]
                .as_array()
                .ok_or("Invalid MCP tool list")?;
            for entry in entries {
                let name = entry["name"]
                    .as_str()
                    .filter(|name| !name.is_empty() && name.len() <= 128)
                    .ok_or("Invalid MCP tool name")?;
                if !names.insert(name.to_owned())
                    || names.len() > 256
                    || !entry["inputSchema"].is_object()
                {
                    return Err("Duplicate, excessive or invalid MCP tool definitions".into());
                }

                let input_schema = serde_json::to_string(&entry["inputSchema"])?;
                size += input_schema.len() + entry["description"].as_str().map_or(0, str::len);
                if input_schema.len() > 65536 || size > LIMIT {
                    return Err("MCP tool schemas exceed metadata limits".into());
                }

                tools.push(domain::McpTool {
                    name: name.to_owned(),
                    description: entry["description"].as_str().unwrap_or("").to_owned(),
                    input_schema,
                });
            }

            cursor = response
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if cursor.is_none() {
                return Ok(tools);
            }
        }

        Err("MCP tool pagination limit exceeded".into())
    }

    pub(super) async fn perform(
        &mut self,
        arguments: Option<domain::McpCallArguments>,
        cancellation: CancellationToken,
    ) -> Result<(Vec<domain::McpTool>, Option<domain::ToolResult>), Error> {
        let mut sent = false;
        let operation = async {
            let tools = self.list().await?;
            let redactions = Arc::clone(&self.redactions);
            let (tools, call) = tokio::task::spawn_blocking(move || -> Result<_, Error> {
                let call = arguments
                    .map(|arguments| -> Result<_, Error> {
                        let tool = tools
                            .iter()
                            .find(|tool| tool.name == arguments.tool)
                            .ok_or("MCP tool is not advertised by the server")?;
                        let schema: Value = serde_json::from_str(&tool.input_schema)?;
                        let input: Value = serde_json::from_str(&arguments.arguments)?;
                        let validator = jsonschema::options()
                            .build(&schema)
                            .map_err(|error| error.to_owned())
                            .context("Invalid or externally referenced MCP input schema")?;
                        if !validator.is_valid(&input) {
                            return Err(
                                "MCP arguments do not match the advertised input schema".into()
                            );
                        }
                        Ok((arguments.tool, input))
                    })
                    .transpose()?;

                let tools = public_tools(tools, &redactions)?;
                Ok((tools, call))
            })
            .await??;
            let Some((name, input)) = call else {
                return Ok((tools, None));
            };

            sent = true;
            let response = self
                .rpc("tools/call", json!({"name":name,"arguments":input}))
                .await?;

            let redactions = Arc::clone(&self.redactions);
            let result = tokio::task::spawn_blocking(move || -> Result<_, Error> {
                let schema: Value = serde_json::from_str(include_str!("content_schema.json"))?;
                let validator = jsonschema::options()
                    .with_format("byte", |value| STANDARD.decode(value).is_ok())
                    .should_validate_formats(true)
                    .build(&schema)
                    .map_err(|error| error.to_owned())
                    .context("Invalid bundled MCP result contract")?;
                if !validator.is_valid(&response) {
                    return Err("Malformed or unsupported MCP tool result content".into());
                }

                let status = if response["isError"] == true {
                    domain::ToolStatus::Failed
                } else {
                    domain::ToolStatus::Succeeded
                };

                let (output, truncated) = match projection::json(response, &redactions, LIMIT, LIMIT * 6 + 2) {
                    Ok(output) => (output, false),
                    Err(_) => ("MCP result content omitted because safe public projection exceeded its limits or produced colliding keys. The validated remote outcome is retained; omission is not a reason to repeat the call.".into(), true),
                };
                Ok(domain::ToolResult {
                    status,
                    output,
                    artifact: None,
                    truncated,
                })
            })
            .await??;

            Ok((tools, Some(result)))
        };

        let result: Result<(Vec<domain::McpTool>, Option<domain::ToolResult>), Error> = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(120), operation) => match result { Ok(result) => result, Err(_) => Err("MCP operation timed out".into()) },
            _ = cancellation.cancelled() => Err("MCP operation cancelled".into()),
        };
        if sent
            && result
                .as_ref()
                .is_err_and(|error| error.is::<RequestRejected>())
        {
            return Ok((Vec::new(), Some(domain::ToolResult::failed("MCP server rejected the tool request. No success was confirmed; verify operation/access and follow documented CLI procedures or ask for configuration. Discovery/read does not establish write authority or an exclusive claim.".into()))));
        }
        if result.is_err() {
            self.kill_group();
            let _ = tokio::time::timeout(Duration::from_secs(2), self.child.kill()).await;
            if sent {
                return Ok((Vec::new(), Some(domain::ToolResult {
                    status: domain::ToolStatus::Unknown,
                    output: "MCP call was sent but its outcome is uncertain. Reconcile before repeating; cancellation is not rollback.".into(),
                    artifact: None, truncated: false,
                })));
            }
        }

        result
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

impl Drop for Client {
    fn drop(&mut self) {
        self.kill_group();
        let _ = self.child.start_kill();
    }
}

fn public_tools(
    mut tools: Vec<domain::McpTool>,
    secrets: &[String],
) -> Result<Vec<domain::McpTool>, Error> {
    let mut remaining = LIMIT - 2;
    for tool in &mut tools {
        if secrets.iter().any(|secret| tool.name.contains(secret)) {
            return Err(
                "MCP tool identity contains a provisioned value; metadata cannot be exposed safely"
                    .into(),
            );
        }

        remaining = remaining
            .checked_sub(64 + projection::encoded_size(&tool.name))
            .ok_or("MCP public metadata exceeds its aggregate budget")?;
        let description = projection::text(
            &tool.description,
            secrets,
            &mut projection::Budget::new(remaining),
        )?;
        tool.description = description.chars().take(4096).collect();
        remaining -= projection::encoded_size(&tool.description);

        let schema: Value = serde_json::from_str(&tool.input_schema)?;
        tool.input_schema = projection::json(schema, secrets, 65536.min(remaining), remaining)?;
        remaining -= projection::encoded_size(&tool.input_schema);
    }
    Ok(tools)
}
