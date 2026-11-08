use crate::tool_io::Error;
use std::sync::Arc;

pub(super) struct ReloadFailure {
    pub cause: Error,
    pub diagnostics: Vec<Arc<dyn std::error::Error + Send + Sync>>,
}

impl std::fmt::Debug for ReloadFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReloadFailure")
            .field("cause", &self.cause)
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}

impl std::fmt::Display for ReloadFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MCP snapshot recovery failed with retained original diagnostics")
    }
}

impl std::error::Error for ReloadFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.cause.as_ref())
    }
}
