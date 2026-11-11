use crate::ProviderFailureKind;
use crate::ProviderId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFailure {
    pub provider_id: ProviderId,
    pub connection_id: String,
    pub model_id: String,

    pub kind: ProviderFailureKind,
    pub retry_at: Option<u64>,
}
