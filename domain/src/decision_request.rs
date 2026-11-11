use crate::DecisionCandidate;
use crate::DecisionResponse;
use crate::Model;
use crate::RoutingEvidence;

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionRequest {
    pub model: Model,
    pub connection_id: String,
    pub candidates: Vec<DecisionCandidate>,
    pub captured_at: u64,
    pub policy: String,
    pub evidence: Vec<(String, Vec<RoutingEvidence>)>,
}

impl DecisionRequest {
    pub const POLICY: &str = "harness-suitability-cost-v1";

    pub fn validate_response(&self, response: &DecisionResponse) -> Result<(), &'static str> {
        let mut keys: Vec<&str> = self.candidates.iter().map(|c| c.key.as_str()).collect();
        keys.push("abstain");
        response.validate(&keys)?;
        self.validate_identity(response)
    }

    pub fn validate_identity(&self, response: &DecisionResponse) -> Result<(), &'static str> {
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
        if self.candidates.is_empty() {
            return Err("Complete native pool is empty");
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
        let mut evidence_keys = std::collections::HashSet::new();
        for (key, facts) in &self.evidence {
            let candidate = self
                .candidates
                .iter()
                .find(|c| &c.key == key)
                .ok_or("Evidence is not bound to a candidate")?;
            if !evidence_keys.insert(key) || facts.len() > 35 {
                return Err("Duplicate or oversized candidate evidence");
            }
            for fact in facts {
                fact.validate()?;
                if fact.requested_model != candidate.model.id
                    || fact.effort != candidate.selection.effort
                    || fact.resolved_model.as_ref().is_some_and(|resolved| {
                        resolved != &candidate.model.id
                            && Some(resolved) != candidate.model.canonical_id.as_ref()
                    })
                {
                    return Err("Evidence cannot borrow another model or effort");
                }
            }
        }
        if self.policy != Self::POLICY || self.connection_id.is_empty() {
            return Err("Invalid routing policy or connection");
        }
        Ok(())
    }
}
