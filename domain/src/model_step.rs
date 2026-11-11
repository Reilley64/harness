use crate::MessageStatus;
use crate::TokenUsage;
use crate::ToolCall;

#[derive(Debug, Clone, PartialEq)]
pub struct ModelStep {
    pub id: String,
    pub output: String,
    pub resolved_model_id: Option<String>,
    pub catalog_canonical_id: Option<String>,
    pub serving_provider: Option<String>,
    pub calls: Vec<ToolCall>,
    pub status: MessageStatus,
    pub usage: Option<TokenUsage>,
    pub reported_cost: Option<f64>,
}
