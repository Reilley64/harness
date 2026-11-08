use crate::McpServerStatus;
use crate::McpTool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpConfiguration {
    pub revision: String,
    pub stale: bool,
    pub error: Option<String>,
    pub servers: Vec<McpServerStatus>,
    pub tools: Vec<McpTool>,
    pub registry_metadata: Option<String>,
}
