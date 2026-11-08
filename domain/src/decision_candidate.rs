use crate::Model;
use crate::ModelSelection;

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionCandidate {
    pub key: String,
    pub selection: ModelSelection,
    pub connection_id: String,
    pub model: Model,
}

impl DecisionCandidate {
    pub fn validate(&self, context_tokens: u64) -> Result<(), &'static str> {
        self.model.validate_worker()?;
        self.model.validate_effort(self.selection.effort)?;
        if let Some(pricing) = &self.model.pricing
            && [pricing.input, pricing.cached_input, pricing.output]
                .into_iter()
                .flatten()
                .any(|price| !price.is_finite() || price < 0.0)
        {
            return Err("Known candidate prices must be finite and nonnegative");
        }
        if self.model.id != self.selection.model_id
            || self.model.provider_id != self.selection.provider_id
            || self
                .model
                .context_window
                .is_none_or(|limit| limit < context_tokens)
            || self.connection_id.is_empty()
        {
            return Err("Candidate identity, connection or required context is incompatible");
        }
        Ok(())
    }
}
