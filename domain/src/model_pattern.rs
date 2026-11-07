use crate::ProviderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPattern {
    value: String,
    provider: String,
    model: String,
}

impl ModelPattern {
    pub fn new(value: String) -> Result<Self, &'static str> {
        if value.len() > 512
            || value
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
            || value.contains('!')
        {
            return Err(
                "Use positive provider/model patterns (only * is a wildcard), at most 512 bytes",
            );
        }
        let (provider, model) = value
            .split_once('/')
            .ok_or("Patterns require provider/model")?;
        if provider.is_empty() || model.is_empty() {
            return Err("Patterns require nonempty provider and model identities");
        }
        Ok(Self {
            provider: provider.into(),
            model: model.into(),
            value,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn permits(&self, provider: &ProviderId, model: &str) -> bool {
        matches(&self.provider, &provider.0) && matches(&self.model, model)
    }
}

fn matches(pattern: &str, value: &str) -> bool {
    let mut parts = pattern.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut remaining) = value.strip_prefix(first) else {
        return false;
    };
    let parts: Vec<_> = parts.collect();
    if parts.is_empty() {
        return remaining.is_empty();
    }
    for part in &parts[..parts.len() - 1] {
        let Some(index) = remaining.find(part) else {
            return false;
        };
        remaining = &remaining[index + part.len()..];
    }
    remaining.ends_with(parts.last().unwrap_or(&""))
}
