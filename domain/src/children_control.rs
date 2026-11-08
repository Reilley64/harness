use crate::JoinMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildrenControl {
    Join {
        id: String,
        mode: JoinMode,
        executions: Vec<String>,
    },
    Background {
        id: String,
    },
    CompleteFollowUp {
        outcomes: Vec<String>,
    },
}
