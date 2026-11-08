use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcRequest, serde::Serialize, serde::Deserialize,
)]
#[request(method = "_harness/mcp", response = Response)]
pub(crate) struct Request {
    pub workspace: PathBuf,
    #[serde(flatten)]
    pub operation: Operation,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Operation {
    Read,
    Reload,
    Save {
        expected_revision: String,
        definition: Definition,
    },
    Remove {
        expected_revision: String,
        name: String,
    },
    Connect {
        name: String,
        revision: String,
    },
    Disconnect {
        name: String,
    },
    Discover {
        name: String,
    },
    Registry {
        query: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Definition {
    name: String,
    #[serde(default = "enabled")]
    enabled: bool,
    #[serde(default)]
    transport: Option<Transport>,
    #[serde(default)]
    environment: BTreeMap<String, String>,
}

fn enabled() -> bool {
    true
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Transport {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    Http {
        url: String,
        #[serde(default)]
        headers: BTreeMap<String, String>,
    },
}

impl From<Definition> for domain::McpDefinition {
    fn from(definition: Definition) -> Self {
        Self {
            name: definition.name,
            enabled: definition.enabled,
            environment: definition.environment,
            transport: match definition.transport {
                Some(Transport::Stdio { command, args }) => {
                    domain::McpTransport::Stdio { command, args }
                }
                Some(Transport::Http { url, headers }) => {
                    domain::McpTransport::Http { url, headers }
                }
                None => domain::McpTransport::Disabled,
            },
        }
    }
}

impl From<domain::McpDefinition> for Definition {
    fn from(definition: domain::McpDefinition) -> Self {
        Self {
            name: definition.name,
            enabled: definition.enabled,
            environment: definition.environment,
            transport: match definition.transport {
                domain::McpTransport::Stdio { command, args } => {
                    Some(Transport::Stdio { command, args })
                }
                domain::McpTransport::Http { url, headers } => {
                    Some(Transport::Http { url, headers })
                }
                domain::McpTransport::Disabled => None,
            },
        }
    }
}

impl From<Operation> for domain::McpConfigurationAction {
    fn from(operation: Operation) -> Self {
        match operation {
            Operation::Read => Self::Read,
            Operation::Reload => Self::Reload,
            Operation::Save {
                expected_revision,
                definition,
            } => Self::Save {
                expected_revision,
                definition: definition.into(),
            },
            Operation::Remove {
                expected_revision,
                name,
            } => Self::Remove {
                expected_revision,
                name,
            },
            Operation::Connect { name, revision } => Self::Connect { name, revision },
            Operation::Disconnect { name } => Self::Disconnect { name },
            Operation::Discover { name } => Self::Discover { name },
            Operation::Registry { query } => Self::Registry { query },
        }
    }
}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcResponse, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct Response {
    revision: String,
    stale: bool,
    error: Option<String>,
    servers: Vec<Server>,
    tools: Vec<Tool>,
    registry_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Server {
    definition: Definition,
    source: String,
    revision: String,
    status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

impl From<domain::McpConfiguration> for Response {
    fn from(configuration: domain::McpConfiguration) -> Self {
        Self {
            revision: configuration.revision,
            stale: configuration.stale,
            error: configuration.error,
            servers: configuration
                .servers
                .into_iter()
                .map(|server| Server {
                    definition: server.definition.into(),
                    source: server.source.as_str().into(),
                    revision: server.revision,
                    status: server.status.as_str().into(),
                })
                .collect(),
            tools: configuration
                .tools
                .into_iter()
                .map(|tool| Tool {
                    name: tool.name,
                    description: tool.description,
                    input_schema: serde_json::from_str(&tool.input_schema).unwrap_or_default(),
                })
                .collect(),
            registry_metadata: configuration
                .registry_metadata
                .and_then(|text| serde_json::from_str(&text).ok()),
        }
    }
}
