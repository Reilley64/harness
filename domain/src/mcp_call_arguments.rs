#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpCallArguments {
    pub server: String,
    pub tool: String,
    pub revision: String,
    pub arguments: String,
}
