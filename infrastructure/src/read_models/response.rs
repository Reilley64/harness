#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Response {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "contextWindow")]
    pub context_window: Option<u64>,
    #[serde(default)]
    pub canonical_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
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

impl From<Response> for domain::Model {
    fn from(dto: Response) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            context_window: dto.context_window,
            canonical_id: dto.canonical_id,
            capabilities: dto.capabilities.into_iter().map(Into::into).collect(),
            provider_id: domain::ProviderId(dto.provider_id),
            efforts: dto.efforts.into_iter().map(Into::into).collect(),
            pricing: dto.pricing.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Capability {
    Chat,
    Decisions,
}

impl From<Capability> for domain::ModelCapability {
    fn from(capability: Capability) -> Self {
        match capability {
            Capability::Chat => Self::Chat,
            Capability::Decisions => Self::Decisions,
        }
    }
}
