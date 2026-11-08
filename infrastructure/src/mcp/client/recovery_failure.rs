use crate::tool_io::Error;

pub(super) struct RecoveryFailure {
    pub original: Error,
    pub recovery: Error,
}

impl std::fmt::Debug for RecoveryFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecoveryFailure")
            .field("original", &self.original)
            .field("recovery", &self.recovery)
            .finish()
    }
}

impl std::fmt::Display for RecoveryFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MCP session recovery failed")
    }
}

impl std::error::Error for RecoveryFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.original.as_ref())
    }
}
