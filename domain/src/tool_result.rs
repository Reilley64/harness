use crate::ToolStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    pub status: ToolStatus,
    pub output: String,
    pub artifact: Option<String>,
    pub truncated: bool,
}

impl ToolResult {
    pub fn success(output: String) -> Self {
        Self {
            status: ToolStatus::Succeeded,
            output,
            artifact: None,
            truncated: false,
        }
    }
    pub fn failed(output: String) -> Self {
        Self {
            status: ToolStatus::Failed,
            output,
            artifact: None,
            truncated: false,
        }
    }
    pub fn cancelled() -> Self {
        Self {
            status: ToolStatus::Cancelled,
            output: "Cancelled; any completed effects are not undone.".into(),
            artifact: None,
            truncated: false,
        }
    }
}
