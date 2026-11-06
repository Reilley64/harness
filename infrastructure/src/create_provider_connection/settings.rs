#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub openai: OpenaiSettings,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenaiSettings {
    pub dynamic_client_id: String,
    pub issuer: String,
    pub resource: String,
    pub scopes: String,
    pub open_browser: bool,
    pub timeout_seconds: u64,
}
