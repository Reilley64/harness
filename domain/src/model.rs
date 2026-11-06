use crate::ModelEffort;
use crate::ModelPricing;
use crate::ProviderId;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
    pub provider_id: ProviderId,
    pub efforts: Vec<ModelEffort>,
    pub pricing: Option<ModelPricing>,
}

impl Model {
    pub fn validate_effort(&self, effort: Option<ModelEffort>) -> Result<(), &'static str> {
        if self.id.trim().is_empty() || self.provider_id.0.trim().is_empty() {
            return Err("A model and provider must be selected");
        }
        if self
            .context_window
            .is_some_and(|limit| limit == 0 || limit > 1_000_000_000)
        {
            return Err("Invalid model context window");
        }
        if effort.is_some_and(|effort| !self.efforts.contains(&effort)) {
            return Err("The model does not support the selected effort");
        }
        Ok(())
    }
}
