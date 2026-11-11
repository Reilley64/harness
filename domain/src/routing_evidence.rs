use crate::ModelEffort;

#[derive(Debug, Clone, PartialEq)]
pub struct RoutingEvidence {
    pub category: String,
    pub metric: String,
    pub value: Option<f64>,
    pub unit: String,

    pub source: String,
    pub source_url: String,
    pub revision: Option<String>,
    pub retrieved_at: Option<u64>,
    pub measured_at: Option<u64>,
    pub published_at: Option<u64>,
    pub effective_at: Option<u64>,
    pub refresh_failed: bool,

    pub requested_model: String,
    pub resolved_model: Option<String>,
    pub serving_provider: Option<String>,
    pub effort: Option<ModelEffort>,
    pub configuration: String,
}

impl RoutingEvidence {
    pub fn status(&self, now: u64) -> &'static str {
        let ttl = if self.category == "benchmark" {
            604_800
        } else {
            86_400
        };
        let observation = if self.category == "benchmark" {
            self.measured_at
        } else {
            self.measured_at.or(self.effective_at).or(self.published_at)
        };
        let Some(retrieved_at) = self.retrieved_at else {
            return "unknown";
        };
        if self.revision.is_none() {
            return "unknown";
        }
        if self.refresh_failed
            || retrieved_at > now
            || now.saturating_sub(retrieved_at) >= ttl
            || [self.measured_at, self.published_at, self.effective_at]
                .into_iter()
                .flatten()
                .any(|at| at > now)
            || observation.is_some_and(|at| now.saturating_sub(at) >= ttl)
        {
            "stale"
        } else if observation.is_none()
            || matches!(self.category.as_str(), "benchmark" | "pricing") && self.value.is_none()
        {
            "unknown"
        } else {
            "fresh"
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(
            self.category.as_str(),
            "catalog" | "capability" | "pricing" | "benchmark"
        ) || self
            .value
            .is_some_and(|n| !n.is_finite() || self.category == "pricing" && n < 0.0)
            || self.category == "pricing" && self.unit != "USD per million tokens"
            || self.source.is_empty()
            || self.source.len() > 1024
            || self.source_url.len() > 2048
            || self
                .revision
                .as_ref()
                .is_some_and(|r| r.is_empty() || r.len() > 512)
            || self.metric.len() > 256
            || self.unit.len() > 256
            || self.configuration.len() > 2048
            || self.requested_model.is_empty()
            || self.requested_model.len() > 512
            || self.resolved_model.as_ref().is_some_and(|s| s.len() > 512)
            || self
                .serving_provider
                .as_ref()
                .is_some_and(|s| s.len() > 512)
        {
            return Err("Invalid sourced routing evidence");
        }
        Ok(())
    }
}
