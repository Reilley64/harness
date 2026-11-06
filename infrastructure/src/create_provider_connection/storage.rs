use crate::storage;

use application::error_context::ResultExt;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use super::AuthenticationError;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Credentials {
    pub version: u32,
    pub host_id: String,
    pub active_client_id: Option<String>,
    pub registrations: BTreeMap<String, Registration>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Registration {
    pub connection_id: String,
    pub issuer: String,
    pub subject: String,
    pub tokens: TokenSet,
    pub saved_at: u64,
    pub expires_at: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub earliest_refresh_at: Option<serde_json::Value>,
}

pub(crate) use crate::storage::lock;

pub(crate) async fn load(path: &Path) -> Result<Credentials, AuthenticationError> {
    match storage::read_private(path, 1_048_576).await {
        Ok(bytes) => {
            let credentials: Credentials =
                tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes))
                    .await
                    .context("Could not process stored credentials")?
                    .context("The credentials file is invalid; it has not been overwritten")?;
            if credentials.version != 1
                || credentials
                    .host_id
                    .strip_prefix("urn:uuid:")
                    .and_then(|id| uuid::Uuid::parse_str(id).ok())
                    .is_none()
            {
                return Err("Unsupported credentials format or invalid host identity".into());
            }
            Ok(credentials)
        }
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|error| error.kind() == ErrorKind::NotFound) =>
        {
            let credentials = Credentials {
                version: 1,
                host_id: format!("urn:uuid:{}", uuid::Uuid::new_v4()),
                active_client_id: None,
                registrations: BTreeMap::new(),
            };
            save(path, &credentials).await?;
            Ok(credentials)
        }
        Err(error) => Err(error),
    }
}

pub(crate) async fn save(
    path: &Path,
    credentials: &Credentials,
) -> Result<(), AuthenticationError> {
    let bytes = serde_json::to_vec_pretty(credentials)?;
    if bytes.len() > 1_048_576 {
        return Err("The credentials file exceeds its size limit".into());
    }
    storage::replace(path, &bytes).await
}
