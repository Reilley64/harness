use crate::ModelPattern;
use crate::ProviderId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelAllowlist {
    patterns: Vec<ModelPattern>,
    customized: bool,
    initialized_providers: Vec<ProviderId>,
}

impl ModelAllowlist {
    pub fn new(patterns: Vec<String>) -> Result<Self, &'static str> {
        Self::restore(patterns, false, Vec::new())
    }

    pub fn restore(
        patterns: Vec<String>,
        customized: bool,
        initialized_providers: Vec<ProviderId>,
    ) -> Result<Self, &'static str> {
        if patterns.len() > 256 {
            return Err("The model allowlist supports at most 256 patterns");
        }
        if initialized_providers.len() > 256 {
            return Err("Too many initialized providers");
        }
        Ok(Self {
            patterns: patterns
                .into_iter()
                .map(ModelPattern::new)
                .collect::<Result<_, _>>()?,
            customized,
            initialized_providers,
        })
    }

    pub fn replace(&mut self, policy: Self) {
        self.patterns = policy.patterns;
        self.customized = true;
    }

    pub fn initialize_provider(&mut self, provider: ProviderId) -> Result<(), &'static str> {
        if self.customized || self.initialized_providers.contains(&provider) {
            return Ok(());
        }
        if self.initialized_providers.len() >= 256 {
            return Err("Too many initialized providers");
        }
        if self.patterns.len() >= 256 {
            return Err("The model allowlist supports at most 256 patterns");
        }
        let pattern = ModelPattern::new(format!("{provider}/*"))?;
        self.patterns.push(pattern);
        self.initialized_providers.push(provider);
        Ok(())
    }

    pub fn patterns(&self) -> &[ModelPattern] {
        &self.patterns
    }

    pub fn is_customized(&self) -> bool {
        self.customized
    }

    pub fn initialized_providers(&self) -> &[ProviderId] {
        &self.initialized_providers
    }

    pub fn permits(&self, provider: &ProviderId, model: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.permits(provider, model))
    }
}
