#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceAction {
    Prepare,
    Open {
        resume: bool,
        rework: Option<String>,
    },
    Inspect,
    Submit {
        commit: String,
    },
}
