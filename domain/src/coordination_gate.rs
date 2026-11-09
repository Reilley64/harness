#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationGate {
    Open,
    NeedsHuman,
    Approval,
    Stopped,
    ResumeRequired,
    Unknown,
}
