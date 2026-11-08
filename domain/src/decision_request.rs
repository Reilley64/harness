use crate::DecisionCandidate;
use crate::DecisionResponse;
use crate::Model;

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionRequest {
    pub model: Model,
    pub connection_id: String,
    pub candidates: Vec<DecisionCandidate>,
    pub captured_at: u64,
    pub policy: String,
}

impl DecisionRequest {
    pub const POLICY: &str = "harness-suitability-cost-v1";

    pub fn validate_response(&self, response: &DecisionResponse) -> Result<(), &'static str> {
        let mut keys: Vec<&str> = self.candidates.iter().map(|c| c.key.as_str()).collect();
        keys.push("abstain");
        response.validate(&keys)?;
        if response
            .resolved_model
            .to_ascii_lowercase()
            .contains("router")
            || !self.model.id.starts_with('~')
                && response.resolved_model != self.model.id
                && self.model.canonical_id.as_ref() != Some(&response.resolved_model)
        {
            return Err("Native serving identity differs from the requested release");
        }
        Ok(())
    }

    pub fn validate_pool(&self, context_tokens: u64) -> Result<(), &'static str> {
        self.model.validate_decider()?;
        if self.candidates.is_empty() || self.candidates.len() > 254 {
            return Err(
                "Unsupported native pool: requires 1..254 pairs plus abstention; no truncation",
            );
        }
        let mut keys = std::collections::HashSet::new();
        for candidate in &self.candidates {
            candidate.validate(context_tokens)?;
            if candidate.key == "abstain"
                || candidate.key.is_empty()
                || !keys.insert(&candidate.key)
            {
                return Err("Invalid opaque candidate keys");
            }
        }
        if self.policy != Self::POLICY || self.connection_id.is_empty() {
            return Err("Invalid routing policy or connection");
        }
        Ok(())
    }
}
