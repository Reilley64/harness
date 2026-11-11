use crate::McpServerState;
use crate::McpTransport;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpDefinition {
    pub name: String,
    pub enabled: bool,
    pub transport: McpTransport,
    pub environment: BTreeMap<String, String>,
}

impl McpDefinition {
    pub fn state(&self, trusted: bool) -> McpServerState {
        if !self.enabled {
            McpServerState::Disabled
        } else if trusted {
            McpServerState::Trusted
        } else {
            McpServerState::Untrusted
        }
    }
}
