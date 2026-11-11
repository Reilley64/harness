use crate::ProviderRetry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageSummary {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub context_tokens: Option<u64>,
    pub model_id: Option<String>,
    pub partial: bool,
    pub provider_retry: Option<ProviderRetry>,
}
