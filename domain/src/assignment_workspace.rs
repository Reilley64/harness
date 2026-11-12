use crate::WorkspaceCheck;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentWorkspace {
    pub assignment_id: String,
    pub path: std::path::PathBuf,
    pub branch: String,
    pub base: String,
    pub tip: String,
    pub submission: Option<String>,
    pub checks: Vec<WorkspaceCheck>,
    pub artifact: String,
}
