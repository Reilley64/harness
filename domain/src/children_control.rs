use crate::JoinMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildrenControl {
    Stop {
        execution_id: String,
    },
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
