use crate::tool_io::Error;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct FileRecord {
    pub version: u32,
    pub servers: Vec<Record>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for FileRecord {
    fn default() -> Self {
        Self {
            version: 1,
            servers: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    pub name: String,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub transport: Option<TransportRecord>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

fn enabled() -> bool {
    true
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum TransportRecord {
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

pub(super) fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

pub(super) fn validate(file: &FileRecord) -> Result<(), Error> {
    if file.version != 1 || file.servers.len() > 128 {
        return Err("Unsupported MCP config version or too many servers".into());
    }
    let mut names = BTreeSet::new();
    for record in &file.servers {
        if !name(&record.name) || !names.insert(&record.name) {
            return Err("Invalid or duplicate MCP server name".into());
        }
        let definition: domain::McpDefinition = record.clone().try_into()?;
        if definition.environment.iter().any(|(key, reference)| {
            !name(key) || !reference.starts_with("HARNESS_MCP_") || !name(reference)
        }) {
            return Err("Environment values must reference provisioned HARNESS_MCP_ variables, never literal credentials".into());
        }
    }
    Ok(())
}

impl TryFrom<Record> for domain::McpDefinition {
    type Error = Error;
    fn try_from(record: Record) -> Result<Self, Error> {
        let transport = match record.transport {
            Some(TransportRecord::Stdio { command, args }) => {
                if command.is_empty()
                    || command.len() > 4096
                    || command.contains('\0')
                    || args.len() > 128
                    || args
                        .iter()
                        .any(|arg| arg.len() > 8192 || arg.contains('\0'))
                {
                    return Err("Invalid MCP command/arguments".into());
                }
                domain::McpTransport::Stdio { command, args }
            }
            Some(TransportRecord::Http { url, headers }) => {
                let parsed = reqwest::Url::parse(&url)?;
                if parsed.scheme() != "https"
                    && !(parsed.scheme() == "http"
                        && matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")))
                    || !parsed.username().is_empty()
                    || parsed.password().is_some()
                    || parsed.query().is_some()
                    || parsed.fragment().is_some()
                    || headers.iter().any(|(key, reference)| {
                        !name(key) || !reference.starts_with("HARNESS_MCP_") || !name(reference)
                    })
                {
                    return Err("MCP HTTP endpoints require HTTPS (or loopback), no URL credentials/query, and provisioned header references".into());
                }
                domain::McpTransport::Http { url, headers }
            }
            None if !record.enabled => domain::McpTransport::Disabled,
            None => return Err("Enabled MCP server requires a transport".into()),
        };
        Ok(Self {
            name: record.name,
            enabled: record.enabled,
            transport,
            environment: record.environment,
        })
    }
}

impl From<domain::McpDefinition> for Record {
    fn from(definition: domain::McpDefinition) -> Self {
        Self {
            name: definition.name,
            enabled: definition.enabled,
            environment: definition.environment,
            transport: match definition.transport {
                domain::McpTransport::Stdio { command, args } => {
                    Some(TransportRecord::Stdio { command, args })
                }
                domain::McpTransport::Http { url, headers } => {
                    Some(TransportRecord::Http { url, headers })
                }
                domain::McpTransport::Disabled => None,
            },
        }
    }
}
