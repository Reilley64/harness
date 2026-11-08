#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Read,
    Bash,
    Edit,
    Write,
    Ls,
    Find,
    Grep,
    AstSearch,
    WebSearch,
    PublishWork,
    McpCall,
    McpConfiguration,
}
impl Tool {
    pub const ALL: [Self; 12] = [
        Self::Read,
        Self::Bash,
        Self::Edit,
        Self::Write,
        Self::Ls,
        Self::Find,
        Self::Grep,
        Self::AstSearch,
        Self::WebSearch,
        Self::PublishWork,
        Self::McpCall,
        Self::McpConfiguration,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Bash => "bash",
            Self::Edit => "edit",
            Self::Write => "write",
            Self::Ls => "ls",
            Self::Find => "find",
            Self::Grep => "grep",
            Self::AstSearch => "ast_search",
            Self::WebSearch => "web_search",
            Self::PublishWork => "publish_work",
            Self::McpCall => "mcp_call",
            Self::McpConfiguration => "mcp_configuration",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tool| tool.as_str() == value)
    }
    pub fn may_mutate(self) -> bool {
        matches!(
            self,
            Self::Bash | Self::Edit | Self::Write | Self::PublishWork | Self::McpCall | Self::McpConfiguration
        )
    }
}
