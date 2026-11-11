#[derive(Debug)]
pub struct Output {
    pub blocked: Option<String>,
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
    pub provider_id: domain::ProviderId,
    pub capabilities: Vec<domain::ModelCapability>,
    pub efforts: Vec<domain::ModelEffort>,
    pub pricing: Option<domain::ModelPricing>,
}

impl From<domain::Model> for Output {
    fn from(model: domain::Model) -> Self {
        Self {
            blocked: None,
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            provider_id: model.provider_id,
            capabilities: model.capabilities,
            efforts: model.efforts,
            pricing: model.pricing,
        }
    }
}
