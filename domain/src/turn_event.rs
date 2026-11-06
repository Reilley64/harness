use crate::ToolCall;
use crate::UsageSummary;

#[derive(Debug, Clone)]
pub enum TurnEvent {
    Text { message_id: String, delta: String },
    Tool { message_id: String, call: ToolCall },
    Usage { usage: UsageSummary },
}
