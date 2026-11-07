#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServerState {
    Disabled,
    Untrusted,
    Trusted,
    HttpExecutionUnavailable,
}

impl McpServerState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Untrusted => "untrusted",
            Self::Trusted => "trusted",
            Self::HttpExecutionUnavailable => "http_execution_unavailable",
        }
    }
}
