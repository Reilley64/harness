#[derive(Debug, Clone, PartialEq)]
pub struct DecisionResponse {
    pub choice: Option<String>,
    pub confidence: Option<f64>,
    pub probabilities: Vec<(String, f64)>,
    pub resolved_model: String,
    pub serving_provider: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: Option<f64>,
}

impl DecisionResponse {
    pub fn validate(&self, keys: &[&str]) -> Result<(), &'static str> {
        if self
            .confidence
            .is_some_and(|n| !n.is_finite() || !(0.0..=1.0).contains(&n))
            || self.cost.is_some_and(|n| !n.is_finite() || n < 0.0)
            || self.input_tokens > 1_000_000_000
            || self.output_tokens > 1_000_000_000
            || self.resolved_model.is_empty()
            || self.resolved_model.len() > 512
            || self
                .serving_provider
                .as_ref()
                .is_some_and(|p| p.is_empty() || p.len() > 512)
        {
            return Err("Invalid native response numerics or provenance");
        }
        if self
            .choice
            .as_ref()
            .is_some_and(|k| !keys.contains(&k.as_str()))
        {
            return Err("Native choice is not a member of the accepted pool");
        }
        let mut seen = std::collections::HashSet::new();
        for (key, value) in &self.probabilities {
            if !keys.contains(&key.as_str())
                || !seen.insert(key)
                || !value.is_finite()
                || !(0.0..=1.0).contains(value)
            {
                return Err("Invalid native probability membership or value");
            }
        }
        if !self.probabilities.is_empty() {
            let sum: f64 = self.probabilities.iter().map(|(_, p)| p).sum();
            if self.probabilities.len() != keys.len() || (sum - 1.0).abs() > 0.0001 {
                return Err("Native probability distribution is incomplete or not normalized");
            }
        }
        Ok(())
    }
}
