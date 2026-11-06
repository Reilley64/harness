#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSelection {
    pub provider_id: crate::ProviderId,
    pub model_id: String,
    pub effort: Option<crate::ModelEffort>,
}
