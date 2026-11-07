use crate::create_model_step;

use std::path::PathBuf;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use config::FileFormat;

use crate::create_provider_connection;
use crate::read_models;
use crate::read_providers;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_path")]
    pub app_data_path: PathBuf,
    #[serde(default)]
    pub tools: ToolsSettings,
    #[serde(default)]
    pub mcp: McpSettings,
    pub replies: create_model_step::Settings,
    pub authentication: create_provider_connection::Settings,
    pub models: read_models::Settings,
    pub providers: read_providers::Settings,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolsSettings {
    pub enabled: bool,
}
impl Default for ToolsSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct McpSettings {
    pub registry_url: String,
}

impl Default for McpSettings {
    fn default() -> Self {
        Self {
            registry_url: "https://registry.modelcontextprotocol.io/v0.1/servers".into(),
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from_str(
                include_str!("../../application.toml"),
                FileFormat::Toml,
            ))
            .add_source(Environment::default().separator("__").try_parsing(true))
            .build()?
            .try_deserialize()
    }
}

pub(crate) fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let path = <PathBuf as serde::Deserialize>::deserialize(deserializer)?;
    if let Ok(relative) = path.strip_prefix("~") {
        let home = std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .ok_or_else(|| {
                serde::de::Error::custom("HOME is required to expand a home-relative path")
            })?;
        Ok(PathBuf::from(home).join(relative))
    } else {
        Ok(path)
    }
}
