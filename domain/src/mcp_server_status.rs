use crate::McpConfigurationSource;
use crate::McpDefinition;
use crate::McpServerState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerStatus {
    pub definition: McpDefinition,
    pub source: McpConfigurationSource,
    pub revision: String,
    pub status: McpServerState,
}
