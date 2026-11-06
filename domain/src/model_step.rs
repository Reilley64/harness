use crate::MessageStatus;
use crate::TokenUsage;
use crate::ToolCall;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelStep {
    pub id: String,
    pub output: String,
    pub calls: Vec<ToolCall>,
    pub status: MessageStatus,
    pub usage: Option<TokenUsage>,
}
