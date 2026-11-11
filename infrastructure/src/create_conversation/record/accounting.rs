#[derive(serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    connection_id: Option<String>,
    outcome: String,
    reported_usd: Option<f64>,
    estimated_usd: Option<f64>,
    pricing_source: Option<String>,
    pricing: Option<PricingRecord>,
}

impl From<domain::GenerationAccounting> for Record {
    fn from(r: domain::GenerationAccounting) -> Self {
        Self {
            connection_id: r.connection_id,
            outcome: r.outcome,
            reported_usd: r.reported_usd,
            estimated_usd: r.estimated_usd,
            pricing_source: r.pricing_source,
            pricing: r.pricing.map(|pricing| PricingRecord {
                input: pricing.input,
                cached_input: pricing.cached_input,
                output: pricing.output,
            }),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PricingRecord {
    input: Option<f64>,
    cached_input: Option<f64>,
    output: Option<f64>,
}
