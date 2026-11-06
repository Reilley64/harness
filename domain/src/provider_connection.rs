use crate::ProviderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConnection {
    pub id: String,
    pub provider_id: ProviderId,
}
