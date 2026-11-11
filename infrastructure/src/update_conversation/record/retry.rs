use crate::conversations;
use crate::conversations::StorageError;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    reply_id: String,
    provider_id: String,
    connection_id: String,
    model_id: String,
    kind: String,
    failures: u32,
    allowance: u32,
    extensions: Vec<String>,
    provider_retry_at: Option<u64>,
    retry_at: Option<u64>,
    waiting: bool,
}

impl TryFrom<Record> for domain::ProviderRetry {
    type Error = StorageError;

    fn try_from(r: Record) -> Result<Self, StorageError> {
        conversations::validate_id(&r.reply_id)?;
        conversations::validate_id(&r.connection_id)?;
        for id in &r.extensions {
            conversations::validate_id(id)?;
        }
        let state = Self {
            reply_id: r.reply_id,
            failure: domain::ProviderFailure {
                provider_id: domain::ProviderId(r.provider_id),
                connection_id: r.connection_id,
                model_id: r.model_id,
                kind: match r.kind.as_str() {
                    "transient" => domain::ProviderFailureKind::Transient,
                    "quota" => domain::ProviderFailureKind::Quota,
                    _ => return Err("Invalid provider wait kind".into()),
                },
                retry_at: r.provider_retry_at,
            },
            failures: r.failures,
            allowance: r.allowance,
            extensions: r.extensions,
            retry_at: r.retry_at,
            waiting: r.waiting,
        };
        state.validate()?;
        Ok(state)
    }
}

impl From<domain::ProviderRetry> for Record {
    fn from(r: domain::ProviderRetry) -> Self {
        Self {
            reply_id: r.reply_id,
            provider_id: r.failure.provider_id.0,
            connection_id: r.failure.connection_id,
            model_id: r.failure.model_id,
            kind: match r.failure.kind {
                domain::ProviderFailureKind::Transient => "transient",
                domain::ProviderFailureKind::Quota => "quota",
                domain::ProviderFailureKind::Permanent => "permanent",
            }
            .into(),
            failures: r.failures,
            allowance: r.allowance,
            extensions: r.extensions,
            provider_retry_at: r.failure.retry_at,
            retry_at: r.retry_at,
            waiting: r.waiting,
        }
    }
}
