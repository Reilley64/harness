#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Record {
    version: u32,
    model: ModelRecord,
    effort: Option<EffortRecord>,
}

impl From<domain::SelectedModel> for Record {
    fn from(selection: domain::SelectedModel) -> Self {
        Self {
            version: 1,
            model: selection.model.into(),
            effort: selection.effort.map(Into::into),
        }
    }
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ModelRecord {
    id: String,
    name: String,
    #[serde(default)]
    context_window: Option<u64>,
    provider_id: String,
    #[serde(default)]
    efforts: Vec<EffortRecord>,
    pricing: Option<PricingRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

impl From<domain::ModelEffort> for EffortRecord {
    fn from(effort: domain::ModelEffort) -> Self {
        match effort {
            domain::ModelEffort::Low => Self::Low,
            domain::ModelEffort::Medium => Self::Medium,
            domain::ModelEffort::High => Self::High,
            domain::ModelEffort::ExtraHigh => Self::ExtraHigh,
        }
    }
}

impl From<domain::Model> for ModelRecord {
    fn from(model: domain::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            provider_id: model.provider_id.0,
            efforts: model.efforts.into_iter().map(Into::into).collect(),
            pricing: model.pricing.map(|pricing| PricingRecord {
                input: pricing.input,
                cached_input: pricing.cached_input,
                output: pricing.output,
            }),
        }
    }
}

impl From<ModelRecord> for domain::Model {
    fn from(dto: ModelRecord) -> Self {
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
