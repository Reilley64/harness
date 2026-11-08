pub struct Output {
    pub configuration: domain::McpConfiguration,
    pub diagnostics: Vec<std::sync::Arc<dyn std::error::Error + Send + Sync>>,
}
