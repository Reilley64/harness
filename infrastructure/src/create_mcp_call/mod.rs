use crate::mcp::Connections;
use crate::tool_io::Error;
use application::ports::PortFuture;
use application::ports::UpdateMcpConfiguration;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    server: String,
    tool: String,
    revision: String,
    arguments: serde_json::Map<String, serde_json::Value>,
}

pub fn decode(text: &str) -> Result<domain::ToolInvocation, Error> {
    let arguments: Arguments = serde_json::from_str(text)?;
    if arguments.server.len() > 64 || arguments.tool.len() > 128 || arguments.revision.len() != 64 {
        return Err("Invalid MCP tool identity or revision".into());
    }
    Ok(domain::ToolInvocation::McpCall(domain::McpCallArguments {
        server: arguments.server,
        tool: arguments.tool,
        revision: arguments.revision,
        arguments: serde_json::to_string(&arguments.arguments)?,
    }))
}

pub fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"mcp_call","strict":false,"description":"Invoke an advertised tool from shared trusted MCP configuration. First use mcp_configuration read and discover to obtain the effective server revision, tool name and JSON input schema. Supply the exact revision. No automatic retry: uncertain effects require reconciliation. Provider credentials are never reused. Discovery does not prove write access or exclusive claim; follow configured CLI/docs when unavailable.","parameters":{"type":"object","properties":{"server":{"type":"string"},"tool":{"type":"string"},"revision":{"type":"string"},"arguments":{"type":"object"}},"required":["server","tool","revision","arguments"],"additionalProperties":false}})
}

pub fn create_mcp_call(
    connections: &Connections,
    configuration: UpdateMcpConfiguration,
) -> impl Fn(
    PathBuf,
    domain::McpCallArguments,
    CancellationToken,
) -> PortFuture<Result<domain::ToolResult, Error>>
+ Send
+ Sync
+ use<> {
    let connections = connections.clone();
    move |workspace, arguments, cancellation| {
        let connections = connections.clone();
        let configuration = configuration.clone();
        Box::pin(async move {
            let workspace = tokio::fs::canonicalize(workspace).await?;
            let snapshot = configuration
                .call(
                    workspace.clone(),
                    domain::McpConfigurationAction::Read,
                    cancellation.clone(),
                )
                .await?;
            let server = snapshot
                .servers
                .iter()
                .find(|server| server.definition.name == arguments.server)
                .ok_or("MCP server is not configured")?;
            if server.revision != arguments.revision {
                return Err("MCP definition changed; discover and review the current revision before execution".into());
            }
            connections
                .call(&workspace, server, arguments, cancellation)
                .await
        })
    }
}
