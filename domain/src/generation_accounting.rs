use crate::ModelPricing;

#[derive(Debug, Clone, PartialEq)]
pub struct GenerationAccounting {
    pub connection_id: Option<String>,
    pub outcome: String,
    pub reported_usd: Option<f64>,
    pub estimated_usd: Option<f64>,
    pub pricing_source: Option<String>,
    pub pricing: Option<ModelPricing>,
}

impl GenerationAccounting {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(
            self.outcome.as_str(),
            "unknown"
                | "completed"
                | "failed"
                | "cancelled"
                | "superseded"
                | "pending"
                | "stopped_unknown"
                | "stopped_before_admission"
                | "legacy_request_unknown"
        ) || [self.reported_usd, self.estimated_usd]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite() || value < 0.0)
            || self
                .pricing_source
                .as_ref()
                .is_some_and(|source| source != "catalog_static_usd_per_million_tokens")
            || self.estimated_usd.is_some()
                && (self.pricing_source.is_none() || self.pricing.is_none())
            || self.pricing.as_ref().is_some_and(|pricing| {
                [pricing.input, pricing.cached_input, pricing.output]
                    .into_iter()
                    .flatten()
                    .any(|value| !value.is_finite() || value < 0.0)
            })
        {
            return Err("Invalid generation accounting");
        }
        Ok(())
    }

    pub fn unknown() -> Self {
        Self {
            connection_id: None,
            outcome: "unknown".into(),
            reported_usd: None,
            estimated_usd: None,
            pricing_source: None,
            pricing: None,
        }
    }
}
