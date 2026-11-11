mod message;
mod projection;
mod recovery_failure;

use super::encoding;
use super::transport::Transport;
use message::Message;
use message::RequestRejected;
use recovery_failure::RecoveryFailure;

use crate::tool_io::Error;
use application::error_context::ResultExt;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const LIMIT: usize = 1_048_576;

pub(super) struct Client {
    transport: Transport,
    next: u64,
    redactions: Arc<[String]>,
    cause: Option<Error>,
}

impl Client {
    pub(super) async fn start(
        transport: Transport,
        redactions: Vec<String>,
        cancellation: CancellationToken,
    ) -> Result<Self, Error> {
        let mut client = Self {
            transport,
            next: 0,
            redactions: Arc::default(),
            cause: None,
        };
        client.protect(redactions).await?;

        if let Err(original) = client.initialize(cancellation).await {
            return match client.transport.close().await {
                Ok(()) => Err(original),
                Err(recovery) => Err(RecoveryFailure { original, recovery }.into()),
            };
        }
        Ok(client)
    }

    async fn initialize(&mut self, cancellation: CancellationToken) -> Result<(), Error> {
        let response = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(30), self.rpc("initialize", json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"harness","version":env!("CARGO_PKG_VERSION")}}))) => result??,
            _ = cancellation.cancelled() => return Err("MCP connection cancelled".into()),
        };
        let version = tokio::task::spawn_blocking(move || -> Result<String, Error> {
            let schema: Value = serde_json::from_str(include_str!("initialize_schema.json"))?;
            let validator = jsonschema::options()
                .build(&schema)
                .map_err(|error| error.to_owned())
                .context("Invalid bundled MCP initialization contract")?;
            validator
                .validate(&response)
                .map_err(|error| error.to_owned())
                .context("Malformed MCP initialization result")?;
            if !matches!(
                response["protocolVersion"].as_str(),
                Some("2025-06-18" | "2025-03-26" | "2024-11-05")
            ) || !response["capabilities"]["tools"].is_object()
            {
                return Err("Unsupported MCP protocol or missing tools capability".into());
            }
            Ok(response["protocolVersion"]
                .as_str()
                .ok_or("Missing MCP protocol version")?
                .to_owned())
        })
        .await??;
        self.transport.negotiate(&version)?;
        self.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .await?;

        Ok(())
    }

    async fn send(&mut self, value: Value) -> Result<(), Error> {
        self.transport.send(value).await
    }

    async fn rpc(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.next += 1;
        let id = self.next;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await?;

        for _ in 0..256 {
            let bytes = self.transport.receive().await?;
            let decoded = tokio::task::spawn_blocking(move || message::parse(bytes, id)).await?;
            let response = match decoded {
                Ok(response) => response,
                Err(error) => {
                    self.transport.finish_response();
                    return Err(error);
                }
            };
            match response {
                Message::Request(request_id) => self.send(json!({"jsonrpc":"2.0","id":request_id,"error":{"code":-32601,"message":"Client capability not supported"}})).await?,
                Message::Notification => {}
                Message::Result(result) => { self.transport.finish_response(); return Ok(result); },
            }
        }

        Err("MCP notification limit exceeded".into())
    }

    async fn list(&mut self) -> Result<Vec<domain::McpTool>, Error> {
        let mut tools = Vec::new();
        let mut names = BTreeSet::new();
        let mut cursor: Option<String> = None;
        let mut size = 0;
        let mut recovered = false;
        for _ in 0..16 {
            let params = cursor
                .as_ref()
                .map_or(json!({}), |cursor| json!({"cursor":cursor}));
            let response = match self.rpc("tools/list", params).await {
                Ok(response) => response,
                Err(error) if self.transport.session_lost() && !recovered => {
                    recovered = true;
                    tools.clear();
                    names.clear();
                    size = 0;
                    let recovery = async {
                        self.initialize(CancellationToken::new()).await?;
                        self.rpc("tools/list", json!({})).await
                    }
                    .await;
                    recovery
                        .map_err(|recovery| RecoveryFailure {
                            original: error,
                            recovery,
                        })
                        .context("MCP read-only discovery failed after session recovery")?
                }
                Err(error) => return Err(error),
            };

            (tools, names, size, cursor) =
                tokio::task::spawn_blocking(move || -> Result<_, Error> {
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
                            return Err(
                                "Duplicate, excessive or invalid MCP tool definitions".into()
                            );
                        }
                        let input_schema = encoding::text(&entry["inputSchema"], 65536)?;
                        let description = match entry.get("description") {
                            Some(value) => value.as_str().ok_or("Invalid MCP tool description")?,
                            None => "",
                        };
                        size += input_schema.len() + description.len();
                        if size > LIMIT {
                            return Err("MCP tool schemas exceed metadata limits".into());
                        }
                        tools.push(domain::McpTool {
                            name: name.to_owned(),
                            description: description.to_owned(),
                            input_schema,
                        });
                    }
                    let cursor = response
                        .get("nextCursor")
                        .map(|value| -> Result<String, Error> {
                            Ok(value
                                .as_str()
                                .filter(|value| !value.is_empty() && value.len() <= 8192)
                                .ok_or("Invalid MCP pagination cursor")?
                                .to_owned())
                        })
                        .transpose()?;
                    Ok((tools, names, size, cursor))
                })
                .await??;
            if cursor.is_none() {
                return Ok(tools);
            }
        }
        Err("MCP tool pagination limit exceeded".into())
    }

    pub(super) fn take_cause(&mut self) -> Option<Error> {
        self.cause.take()
    }

    pub(super) fn is_http(&self) -> bool {
        self.transport.is_http()
    }

    async fn protect(&mut self, secrets: Vec<String>) -> Result<(), Error> {
        if !secrets.is_empty() {
            let previous = Arc::clone(&self.redactions);
            self.redactions = tokio::task::spawn_blocking(move || -> Result<Arc<[String]>, Error> {
                let mut combined = previous.to_vec();
                combined.extend(secrets);
                combined.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
                combined.dedup();
                if combined.len() > 256 { return Err("MCP credential-redaction lifetime reached its bound; reconnect before new work".into()); }
                Ok(combined.into())
            }).await??;
        }
        Ok(())
    }

    async fn project(&mut self, response: Value) -> Result<domain::ToolResult, Error> {
        let redactions = Arc::clone(&self.redactions);
        let (result, cause) = tokio::task::spawn_blocking(move || -> Result<_, Error> {
            let schema: Value = serde_json::from_str(include_str!("content_schema.json"))?;
            let validator = jsonschema::options()
                .with_format("byte", |value| STANDARD.decode(value).is_ok())
                .should_validate_formats(true)
                .build(&schema)
                .map_err(|error| error.to_owned())
                .context("Invalid bundled MCP result contract")?;
            validator.validate(&response).map_err(|error| error.to_owned()).context("Malformed or unsupported MCP tool result content")?;

            let status = if response["isError"] == true {
                domain::ToolStatus::Failed
            } else {
                domain::ToolStatus::Succeeded
            };

            let (output, truncated, cause) = match projection::json(response, &redactions, LIMIT, LIMIT * 6 + 2) {
                Ok(output) => (output, false, None),
                Err(error) => ("MCP result content omitted because safe public projection exceeded its limits or produced colliding keys. The validated remote outcome is retained; omission is not a reason to repeat the call.".into(), true, Some(error)),
            };
            Ok((domain::ToolResult {
                status,
                output,
                artifact: None,
                truncated,
            }, cause))
        })
        .await??;

        self.cause = cause;
        Ok(result)
    }

    pub(super) async fn perform(
        &mut self,
        arguments: Option<domain::McpCallArguments>,
        cancellation: CancellationToken,
    ) -> Result<(Vec<domain::McpTool>, Option<domain::ToolResult>), Error> {
        let secrets = self.transport.prepare(cancellation.clone()).await?;
        self.protect(secrets).await?;
        if self.transport.session_lost() {
            self.initialize(cancellation.clone()).await?;
        }
        self.transport.reset_call_sent();
        let operation = async {
            let tools = self.list().await?;
            if let Some(session) = self.transport.session_identity() {
                self.protect(vec![session]).await?;
            }
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
                        validator
                            .validate(&input)
                            .map_err(|error| error.to_owned())
                            .context("MCP arguments do not match the advertised input schema")?;
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

            let response = self
                .rpc("tools/call", json!({"name":name,"arguments":input}))
                .await?;

            Ok((tools, Some(response)))
        };

        let received: Result<(Vec<domain::McpTool>, Option<Value>), Error> = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(120), operation) => match result { Ok(result) => result, Err(error) => Err(error).context("MCP operation timed out").map_err(Into::into) },
            _ = cancellation.cancelled() => Err("MCP operation cancelled".into()),
        };
        let mut result = match received {
            Ok((tools, Some(response))) => self
                .project(response)
                .await
                .map(|result| (tools, Some(result))),
            Ok((tools, None)) => Ok((tools, None)),
            Err(error) => Err(error),
        };
        let sent = self.transport.call_sent();
        if sent
            && result
                .as_ref()
                .is_err_and(|error| error.is::<RequestRejected>())
        {
            self.cause = result.err();
            return Ok((Vec::new(), Some(domain::ToolResult::failed("MCP server rejected the tool request. No success was confirmed; verify operation/access and follow documented CLI procedures or ask for configuration. Discovery/read does not establish write authority or an exclusive claim.".into()))));
        }
        if result.is_err() {
            if let Err(recovery) = self.transport.close().await
                && let Err(original) = result
            {
                result = Err(RecoveryFailure { original, recovery }.into());
            }
            if sent {
                self.cause = result.err();
                return Ok((Vec::new(), Some(domain::ToolResult {
                    status: domain::ToolStatus::Unknown,
                    output: "MCP call transmission was attempted but its outcome is uncertain. Reconcile before repeating; cancellation is not rollback.".into(),
                    artifact: None, truncated: false,
                })));
            }
        }

        result
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
