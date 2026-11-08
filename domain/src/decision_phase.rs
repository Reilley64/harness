#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionPhase {
    Prepared,
    Pending,
    Selected,
    Fallback,
    Configuration,
    Unsupported,
    Stopped,
}
