#[derive(Debug)]
pub struct Output {
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
    pub provider_id: domain::ProviderId,
    pub efforts: Vec<domain::ModelEffort>,
    pub pricing: Option<domain::ModelPricing>,
}

impl From<domain::Model> for Output {
    fn from(model: domain::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            provider_id: model.provider_id,
            efforts: model.efforts,
            pricing: model.pricing,
        }
    }
}
