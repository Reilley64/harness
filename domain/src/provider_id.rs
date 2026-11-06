use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ProviderId(pub String);

impl From<String> for ProviderId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl Display for ProviderId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
