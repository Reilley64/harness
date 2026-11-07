#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    NeedsHuman,
    Stopped,
    Failed,
    Interrupted,
}
