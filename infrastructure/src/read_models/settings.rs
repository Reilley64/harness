#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub url: String,
    pub memory_ttl_seconds: u64,
}
