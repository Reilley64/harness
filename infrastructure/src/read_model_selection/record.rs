#[derive(serde::Deserialize)]
pub(super) struct Record {
    version: u32,
    model: ModelRecord,
    effort: Option<EffortRecord>,
}

impl TryFrom<Record> for domain::SelectedModel {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(dto: Record) -> Result<Self, Self::Error> {
        if dto.version != 1 {
            return Err(
                "Unsupported saved model selection format; choose a model with /model".into(),
            );
        }
        Ok(Self::new(dto.model.into(), dto.effort.map(Into::into))?)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ModelRecord {
    id: String,
    name: String,
    #[serde(default)]
    context_window: Option<u64>,
    #[serde(default)]
    canonical_id: Option<String>,
    #[serde(default)]
    capabilities: Vec<CapabilityRecord>,
    provider_id: String,
    #[serde(default)]
    efforts: Vec<EffortRecord>,
    pricing: Option<PricingRecord>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum EffortRecord {
    Low,
    Medium,
    High,
    ExtraHigh,
}

impl From<EffortRecord> for domain::ModelEffort {
    fn from(effort: EffortRecord) -> Self {
        match effort {
            EffortRecord::Low => Self::Low,
            EffortRecord::Medium => Self::Medium,
            EffortRecord::High => Self::High,
            EffortRecord::ExtraHigh => Self::ExtraHigh,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PricingRecord {
    input: Option<f64>,
    cached_input: Option<f64>,
    output: Option<f64>,
}

impl From<PricingRecord> for domain::ModelPricing {
    fn from(pricing: PricingRecord) -> Self {
        Self {
            input: pricing.input,
            cached_input: pricing.cached_input,
            output: pricing.output,
        }
    }
}

impl From<ModelRecord> for domain::Model {
    fn from(dto: ModelRecord) -> Self {
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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum CapabilityRecord {
    Chat,
    Decisions,
}

impl From<CapabilityRecord> for domain::ModelCapability {
    fn from(capability: CapabilityRecord) -> Self {
        match capability {
            CapabilityRecord::Chat => Self::Chat,
            CapabilityRecord::Decisions => Self::Decisions,
        }
    }
}
