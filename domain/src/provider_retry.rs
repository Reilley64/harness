use crate::ProviderFailure;
use crate::ProviderFailureKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRetry {
    pub reply_id: String,
    pub failure: ProviderFailure,
    pub failures: u32,
    pub allowance: u32,
    pub extensions: Vec<String>,
    pub retry_at: Option<u64>,
    pub waiting: bool,
}

impl ProviderRetry {
    pub const MAX_RETRY_AT: u64 = 253_402_300_799_999;

    pub fn failed(
        previous: Option<Self>,
        reply_id: String,
        failure: ProviderFailure,
        now: u64,
        jitter: u64,
    ) -> Result<Self, &'static str> {
        if failure.kind == ProviderFailureKind::Permanent {
            return Err("Permanent provider failures cannot enter automatic retry");
        }
        let mut state = previous.unwrap_or_else(|| Self {
            reply_id: reply_id.clone(),
            failure: failure.clone(),
            failures: 0,
            allowance: 2,
            extensions: Vec::new(),
            retry_at: None,
            waiting: true,
        });
        if state.reply_id != reply_id
            || state.failure.provider_id != failure.provider_id
            || state.failure.connection_id != failure.connection_id
            || state.failure.model_id != failure.model_id
        {
            return Err("Provider retry ownership changed; inspect the original attempt");
        }
        state.failures = state
            .failures
            .checked_add(1)
            .ok_or("Provider failure count overflow")?;
        state.retry_at = match failure.kind {
            ProviderFailureKind::Transient if state.failures <= state.allowance => {
                let ceiling = 1_000u64
                    .saturating_mul(1u64 << state.failures.min(6))
                    .min(60_000);
                Some(failure.retry_at.unwrap_or_else(|| {
                    now.saturating_add(ceiling / 2 + jitter % (ceiling / 2 + 1))
                }))
            }
            ProviderFailureKind::Quota => failure.retry_at,
            _ => None,
        };
        state.failure = failure;
        state.waiting = true;
        state.validate()?;
        Ok(state)
    }

    pub fn extend(&mut self, id: String, additional: u32) -> Result<(), &'static str> {
        if self.extensions.contains(&id) {
            return Err("This extension identity has already been used; inspect current allowance");
        }
        if self.failure.kind != ProviderFailureKind::Transient
            || self.retry_at.is_some()
            || self.failures <= self.allowance
            || additional == 0
            || additional > 64
            || self.extensions.len() >= 64
        {
            return Err(
                "Only an exhausted transient retry allowance can be explicitly extended by 1..64",
            );
        }
        self.allowance = self
            .allowance
            .checked_add(additional)
            .ok_or("Retry allowance overflow")?;
        self.extensions.push(id);
        self.retry_at = self.failure.retry_at.or(Some(0));
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        if self.retry_at.is_none() {
            if self.failure.kind != ProviderFailureKind::Quota {
                return Err("Transient retries exhausted; a trusted human extension is required");
            }
            self.retry_at = Some(0);
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.retry_at.is_some_and(|time| time > Self::MAX_RETRY_AT)
            || self
                .failure
                .retry_at
                .is_some_and(|time| time > Self::MAX_RETRY_AT)
            || self.failure.provider_id.0.is_empty()
            || self.reply_id.is_empty()
            || self.failure.connection_id.is_empty()
            || self.failure.model_id.is_empty()
            || self.failures == 0
            || self.allowance < 2
            || self.allowance > 4_098
            || self.extensions.len() > 64
            || self
                .extensions
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != self.extensions.len()
            || self.failure.kind == ProviderFailureKind::Permanent
            || (self.failure.kind == ProviderFailureKind::Transient
                && self.failures > self.allowance
                && self.retry_at.is_some())
        {
            return Err("Invalid durable provider retry authority");
        }
        Ok(())
    }
}
