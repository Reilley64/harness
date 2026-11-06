use crate::ProviderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub id: ProviderId,
    pub name: String,
}
