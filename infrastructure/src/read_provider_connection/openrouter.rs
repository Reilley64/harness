use std::path::Path;

use crate::storage;

#[derive(serde::Deserialize)]
struct Record {
    version: u32,
    connection_id: String,
    issuer: String,
    key: String,
}

pub(super) async fn read(
    path: &Path,
) -> Result<domain::ProviderConnection, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = storage::read_private(path, 1_048_576).await?;
    tokio::task::spawn_blocking(move || {
        let record: Record = serde_json::from_slice(&bytes)?;
        if record.version != 1
            || uuid::Uuid::parse_str(&record.connection_id).is_err()
            || record.key.is_empty()
            || record.key.len() > 16384
            || record.key.chars().any(char::is_control)
            || reqwest::Url::parse(&record.issuer).is_err()
        {
            return Err("Invalid stored OpenRouter connection".into());
        }
        Ok(domain::ProviderConnection {
            id: record.connection_id,
            provider_id: domain::ProviderId("openrouter".into()),
        })
    })
    .await?
}
