use crate::ModelPattern;
use crate::ProviderId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelAllowlist {
    patterns: Vec<ModelPattern>,
}

impl ModelAllowlist {
    pub fn new(patterns: Vec<String>) -> Result<Self, &'static str> {
        if patterns.len() > 256 {
            return Err("The model allowlist supports at most 256 patterns");
        }
        Ok(Self {
            patterns: patterns
                .into_iter()
                .map(ModelPattern::new)
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn patterns(&self) -> &[ModelPattern] {
        &self.patterns
    }

    pub fn permits(&self, provider: &ProviderId, model: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.permits(provider, model))
    }
}
