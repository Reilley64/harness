#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpConfigurationSource {
    App,
    Repository,
}

impl McpConfigurationSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Repository => "repository",
        }
    }
}
