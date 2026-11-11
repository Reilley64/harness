use crate::GenerationAccounting;
use crate::TokenUsage;

#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntry {
    pub id: String,
    pub source_id: String,
    pub source_revision: u64,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub work_id: Option<String>,
    pub category: String,

    pub provider_id: String,
    pub connection_id: Option<String>,
    pub model_id: String,
    pub billing: String,
    pub admission: String,

    pub tokens: Option<TokenUsage>,
    pub accounting: GenerationAccounting,
}
