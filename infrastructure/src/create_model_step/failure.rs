use super::ModelStepError;
use chrono::DateTime;
use chrono::NaiveDateTime;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(super) fn now() -> Result<u64, ModelStepError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

pub(super) fn retry_at(value: Option<&str>, now: u64) -> Option<u64> {
    let value = value?.trim();
    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        let time = now.checked_add(value.parse::<u64>().ok()?.checked_mul(1_000)?)?;
        return (time <= domain::ProviderRetry::MAX_RETRY_AT).then_some(time);
    }
    let date = DateTime::parse_from_rfc2822(value)
        .map(|date| date.timestamp_millis())
        .or_else(|_| {
            NaiveDateTime::parse_from_str(value, "%A, %d-%b-%y %H:%M:%S GMT")
                .map(|date| date.and_utc().timestamp_millis())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(value, "%a %b %e %H:%M:%S %Y")
                .map(|date| date.and_utc().timestamp_millis())
        })
        .ok()?;
    let date = u64::try_from(date).ok()?;
    (date >= now && date <= domain::ProviderRetry::MAX_RETRY_AT).then_some(date)
}

pub(super) fn event_kind(event: &serde_json::Value) -> domain::ProviderFailureKind {
    kind(
        200,
        event
            .pointer("/response/error/code")
            .or_else(|| event.pointer("/error/code"))
            .or_else(|| event.get("code"))
            .and_then(serde_json::Value::as_str),
    )
}

pub(super) fn response_kind(
    provider: &str,
    status: u16,
    body: &serde_json::Value,
) -> domain::ProviderFailureKind {
    let code = body
        .pointer("/error/code")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            body.pointer("/error/type")
                .and_then(serde_json::Value::as_str)
        });
    if status == 402
        && provider == "openrouter"
        && body
            .pointer("/error/code")
            .and_then(serde_json::Value::as_u64)
            == Some(402)
        && body
            .pointer("/error/message")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|message| message.trim_start().starts_with("Insufficient credits."))
    {
        return domain::ProviderFailureKind::Quota;
    }
    kind(status, code)
}

pub(super) fn kind(status: u16, code: Option<&str>) -> domain::ProviderFailureKind {
    if matches!(status, 400 | 401 | 403 | 404 | 422) {
        return domain::ProviderFailureKind::Permanent;
    }
    match code {
        Some(
            "insufficient_quota" | "quota_exceeded" | "insufficient_credits" | "credits_exhausted",
        ) => domain::ProviderFailureKind::Quota,
        Some(
            "invalid_api_key"
            | "authentication_error"
            | "invalid_request_error"
            | "context_length_exceeded"
            | "insufficient_budget"
            | "budget_exceeded"
            | "max_tokens_exceeded",
        ) => domain::ProviderFailureKind::Permanent,
        Some(
            "rate_limit_exceeded"
            | "rate_limit_error"
            | "server_error"
            | "overloaded_error"
            | "temporarily_unavailable",
        ) if !matches!(status, 400 | 401 | 403 | 404 | 422) => {
            domain::ProviderFailureKind::Transient
        }
        _ if status == 429 || status == 408 || (500..=599).contains(&status) => {
            domain::ProviderFailureKind::Transient
        }
        _ => domain::ProviderFailureKind::Permanent,
    }
}

pub(super) async fn body(
    response: &mut reqwest::Response,
) -> Result<serde_json::Value, ModelStepError> {
    let mut body = Vec::new();
    while let Some(bytes) = response.chunk().await? {
        if body.len().saturating_add(bytes.len()) > 1_048_576 {
            return Err("Provider failure response exceeds 1 MiB".into());
        }
        body.extend_from_slice(&bytes);
    }
    tokio::task::spawn_blocking(move || {
        Ok(serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
    })
    .await?
}
