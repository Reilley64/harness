#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Response {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "contextWindow")]
    pub context_window: Option<u64>,
    pub provider_id: String,
    #[serde(default)]
    pub efforts: Vec<Effort>,
    pub pricing: Option<Pricing>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Effort {
    Low,
    Medium,
    High,
    ExtraHigh,
}

impl From<Effort> for domain::ModelEffort {
    fn from(effort: Effort) -> Self {
        match effort {
            Effort::Low => Self::Low,
            Effort::Medium => Self::Medium,
            Effort::High => Self::High,
            Effort::ExtraHigh => Self::ExtraHigh,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Pricing {
    pub input: Option<f64>,
    pub cached_input: Option<f64>,
    pub output: Option<f64>,
}

impl From<Pricing> for domain::ModelPricing {
    fn from(pricing: Pricing) -> Self {
        Self {
            input: pricing.input,
            cached_input: pricing.cached_input,
            output: pricing.output,
        }
    }
}

impl From<domain::ModelEffort> for Effort {
    fn from(effort: domain::ModelEffort) -> Self {
        match effort {
            domain::ModelEffort::Low => Self::Low,
            domain::ModelEffort::Medium => Self::Medium,
            domain::ModelEffort::High => Self::High,
            domain::ModelEffort::ExtraHigh => Self::ExtraHigh,
        }
    }
}

impl From<domain::Model> for Response {
    fn from(model: domain::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            provider_id: model.provider_id.0,
            efforts: model.efforts.into_iter().map(Into::into).collect(),
            pricing: model.pricing.map(|pricing| Pricing {
                input: pricing.input,
                cached_input: pricing.cached_input,
                output: pricing.output,
            }),
        }
    }
}

impl From<Response> for domain::Model {
    fn from(dto: Response) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            context_window: dto.context_window,
            provider_id: domain::ProviderId(dto.provider_id),
            efforts: dto.efforts.into_iter().map(Into::into).collect(),
            pricing: dto.pricing.map(Into::into),
        }
    }
}
