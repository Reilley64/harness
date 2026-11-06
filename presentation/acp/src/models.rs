#[derive(
    Debug,
    Clone,
    Default,
    agent_client_protocol::JsonRpcRequest,
    serde::Serialize,
    serde::Deserialize,
)]
#[request(method = "_harness/models", response = ModelsResponse)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModelsRequest {}

#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcResponse, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct ModelsResponse {
    pub models: Vec<ModelSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelSummary {
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
    pub provider_id: String,
    pub efforts: Vec<ModelEffort>,
    pub pricing: Option<ModelPricing>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ModelEffort {
    Low,
    Medium,
    High,
    ExtraHigh,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelPricing {
    pub input: Option<f64>,
    pub cached_input: Option<f64>,
    pub output: Option<f64>,
}

impl From<domain::Model> for ModelSummary {
    fn from(model: domain::Model) -> Self {
        Self::from(application::use_cases::read_models::Output {
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            provider_id: model.provider_id,
            efforts: model.efforts,
            pricing: model.pricing,
        })
    }
}

impl From<domain::ModelEffort> for ModelEffort {
    fn from(effort: domain::ModelEffort) -> Self {
        match effort {
            domain::ModelEffort::Low => Self::Low,
            domain::ModelEffort::Medium => Self::Medium,
            domain::ModelEffort::High => Self::High,
            domain::ModelEffort::ExtraHigh => Self::ExtraHigh,
        }
    }
}

impl From<ModelEffort> for domain::ModelEffort {
    fn from(effort: ModelEffort) -> Self {
        match effort {
            ModelEffort::Low => Self::Low,
            ModelEffort::Medium => Self::Medium,
            ModelEffort::High => Self::High,
            ModelEffort::ExtraHigh => Self::ExtraHigh,
        }
    }
}

impl From<application::use_cases::read_models::Output> for ModelSummary {
    fn from(model: application::use_cases::read_models::Output) -> Self {
        Self {
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            provider_id: model.provider_id.0,
            efforts: model
                .efforts
                .into_iter()
                .map(|effort| match effort {
                    domain::ModelEffort::Low => ModelEffort::Low,
                    domain::ModelEffort::Medium => ModelEffort::Medium,
                    domain::ModelEffort::High => ModelEffort::High,
                    domain::ModelEffort::ExtraHigh => ModelEffort::ExtraHigh,
                })
                .collect(),
            pricing: model.pricing.map(|pricing| ModelPricing {
                input: pricing.input,
                cached_input: pricing.cached_input,
                output: pricing.output,
            }),
        }
    }
}
