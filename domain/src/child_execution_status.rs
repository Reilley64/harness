#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildExecutionStatus {
    Held,
    Running,
    Completed,
    NeedsHuman,
    Failed,
    Stopped,
    Unknown,
}

impl ChildExecutionStatus {
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Held | Self::Running)
    }
}
