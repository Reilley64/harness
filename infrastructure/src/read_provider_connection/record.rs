use application::error_context::ResultExt;

use crate::storage;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

type AuthenticationError = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Record {
    pub version: u32,
    pub host_id: String,
    pub active_client_id: Option<String>,
    pub registrations: BTreeMap<String, Registration>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Registration {
    pub connection_id: String,
    pub issuer: String,
    pub subject: String,
    pub tokens: TokenSet,
    pub saved_at: u64,
    pub expires_at: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub earliest_refresh_at: Option<serde_json::Value>,
}

pub(super) async fn load(path: &Path) -> Result<Record, AuthenticationError> {
    match storage::read_private(path, 1_048_576).await {
        Ok(bytes) => {
            let credentials: Record =
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
            Err("Sign in with /auth before starting a conversation".into())
        }
        Err(error) => Err(error),
    }
}
