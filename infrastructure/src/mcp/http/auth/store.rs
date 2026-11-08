use super::super::endpoint;
use super::super::failure;
use super::callback::validate_redirect;
use super::oauth_destination;
use super::valid_scope;
use crate::storage;
use crate::tool_io::Error;
use application::error_context::ResultExt;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio_util::sync::CancellationToken;

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    pub version: u32,
    pub generation: u64,
    pub pending: Option<String>,
    pub grant: Option<Grant>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Grant {
    pub generation: u64,
    pub resource: String,
    pub issuer: String,
    pub client_id: String,
    pub redirect: String,
    pub token_endpoint: String,
    pub account: Option<String>,
    pub access: String,
    pub refresh: Option<String>,
    #[serde(default)]
    pub identity_token: Option<String>,
    pub expires_at: u64,
    pub scope: Option<String>,
    #[serde(default)]
    pub required_scope: Option<String>,
    pub unusable: bool,
}

pub(super) async fn load(path: &Path) -> Result<Record, Error> {
    match tokio::fs::symlink_metadata(path).await {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Record {
                version: 1,
                ..Record::default()
            });
        }
        Err(error) => return Err(error.into()),
        Ok(_) => {}
    }
    let bytes = storage::read_private(path, 262144).await.context(
        "MCP private credentials are inaccessible; preserve the record and check permissions",
    )?;
    tokio::task::spawn_blocking(move || -> Result<Record, Error> {
        let record: Record =
            serde_json::from_slice(&bytes).context("Invalid MCP private credential record")?;
        if record.version != 1 {
            return Err(failure("Unsupported MCP private credential version"));
        }
        if record
            .pending
            .as_ref()
            .is_some_and(|pending| pending.len() != 36 || uuid::Uuid::parse_str(pending).is_err())
        {
            return Err(failure("Invalid saved MCP authorization lease"));
        }
        if let Some(grant) = &record.grant {
            if grant.generation == 0 || grant.generation > record.generation {
                return Err(failure("Invalid saved MCP grant generation"));
            }
            endpoint(&grant.resource)?;
            endpoint(&grant.issuer)?;
            oauth_destination(&grant.token_endpoint)?;
            validate_redirect(&grant.redirect)?;
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            if grant.expires_at == 0
                || grant.expires_at > now.saturating_add(31536000)
                || grant
                    .scope
                    .as_ref()
                    .is_some_and(|scope| !valid_scope(scope))
                || grant
                    .required_scope
                    .as_ref()
                    .is_some_and(|scope| !valid_scope(scope))
            {
                return Err(failure("Invalid saved MCP grant expiry or scope"));
            }
            if !access(&grant.access)
                || grant.refresh.as_ref().is_some_and(|value| !secret(value))
                || grant
                    .identity_token
                    .as_ref()
                    .is_some_and(|value| !secret(value))
                || grant.resource.len() > 8192
                || grant.issuer.len() > 8192
                || !secret(&grant.client_id)
                || grant.client_id.len() > 8192
                || grant
                    .account
                    .as_ref()
                    .is_some_and(|value| value.is_empty() || value.len() > 8192)
            {
                return Err(failure("Invalid saved MCP credential bounds"));
            }
        }
        Ok(record)
    })
    .await?
}

pub(super) fn secret(value: &str) -> bool {
    !value.is_empty() && value.len() <= 16384 && !value.chars().any(char::is_control)
}

pub(super) fn access(value: &str) -> bool {
    secret(value)
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/' | b'=')
        })
}

pub(super) async fn save(path: &Path, record: Record) -> Result<(), Error> {
    let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Error> {
        let bytes = serde_json::to_vec(&record)?;
        if bytes.len() > 262144 {
            return Err(failure("MCP private credential record exceeds its bound"));
        }
        Ok(bytes)
    })
    .await??;
    storage::replace(path, &bytes)
        .await
        .context("MCP credentials could not be committed; authentication has not completed")?;
    Ok(())
}

pub(super) async fn lock(
    path: &Path,
    cancellation: &CancellationToken,
) -> Result<std::fs::File, Error> {
    for attempt in 0..100 {
        if cancellation.is_cancelled() {
            return Err(failure("MCP credential wait cancelled before admission"));
        }
        match storage::lock(path).await {
            Ok(guard) => return Ok(guard),
            Err(error)
                if error
                    .source()
                    .and_then(|source| source.downcast_ref::<std::fs::TryLockError>())
                    .is_some_and(|error| matches!(error, std::fs::TryLockError::WouldBlock)) =>
            {
                if attempt == 99 {
                    return Err(error).context("MCP private storage remained busy; preserve pending work and retry explicit configuration after the other operation finishes").map_err(Into::into);
                }
                tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(error).context("MCP credential wait cancelled before admission").map_err(Into::into),
                    _ = tokio::time::sleep(Duration::from_millis(20)) => {}
                }
            }
            Err(error) => return Err(error),
        }
    }
    Err(failure("MCP private storage lock admission failed"))
}

pub(super) async fn begin(
    path: &Path,
    invalidate: bool,
    expected_access: Option<&str>,
    expected_generation: Option<u64>,
    cancellation: CancellationToken,
) -> Result<(String, Option<Grant>, u64), Error> {
    let _lock = lock(path, &cancellation)
        .await
        .context("MCP credential update is busy or inaccessible")?;
    let mut record = load(path).await?;
    if invalidate
        && (record.pending.is_some()
            || Some(record.generation) != expected_generation
            || record.grant.as_ref().map(|grant| grant.access.as_str()) != expected_access)
    {
        return Err(failure(
            "MCP refresh ownership changed; no token exchange started",
        ));
    }
    let pending = uuid::Uuid::new_v4().to_string();
    record.generation = record
        .generation
        .checked_add(1)
        .ok_or_else(|| failure("MCP credential generation exhausted"))?;
    record.pending = Some(pending.clone());
    let generation = record.generation;
    let (record, previous) = tokio::task::spawn_blocking(move || {
        let previous = record.grant.clone();
        if invalidate && let Some(grant) = &mut record.grant {
            grant.unusable = true;
        }
        (record, previous)
    })
    .await?;
    if cancellation.is_cancelled() {
        return Err(failure(
            "MCP authorization cancelled before private write admission",
        ));
    }
    save(path, record).await?;
    Ok((pending, previous, generation))
}

pub(super) async fn complete(path: &Path, pending: &str, mut grant: Grant) -> Result<(), Error> {
    let _lock = lock(path, &CancellationToken::new())
        .await
        .context("MCP credential completion is busy or inaccessible")?;
    let mut record = load(path).await?;
    if record.pending.as_deref() != Some(pending) {
        return Err(failure(
            "MCP authorization completion is stale; a newer grant or disconnect owns this identity",
        ));
    }
    record.pending = None;
    grant.generation = record.generation;
    record.grant = Some(grant);
    save(path, record).await
}

pub(super) async fn abandon(path: &Path, pending: &str) -> Result<(), Error> {
    let _lock = lock(path, &CancellationToken::new())
        .await
        .context("MCP credential recovery is busy or inaccessible")?;
    let mut record = load(path).await?;
    if record.pending.as_deref() == Some(pending) {
        record.pending = None;
        save(path, record).await?;
    }
    Ok(())
}
