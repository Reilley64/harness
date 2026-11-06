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
}
impl Tool {
    pub const ALL: [Self; 9] = [
        Self::Read,
        Self::Bash,
        Self::Edit,
        Self::Write,
        Self::Ls,
        Self::Find,
        Self::Grep,
        Self::AstSearch,
        Self::WebSearch,
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
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tool| tool.as_str() == value)
    }
    pub fn may_mutate(self) -> bool {
        matches!(self, Self::Bash | Self::Edit | Self::Write)
    }
}
