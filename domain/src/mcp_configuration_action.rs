use crate::McpDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpConfigurationAction {
    Read,
    Reload,
    Save {
        expected_revision: String,
        definition: McpDefinition,
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
