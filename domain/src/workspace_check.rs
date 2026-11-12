use crate::ToolStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceCheck {
    pub command: String,
    pub status: ToolStatus,
    pub output: String,
}
