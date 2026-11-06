#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    InProgress,
    Completed,
    Cancelled,
    Failed,
    Interrupted,
    Incomplete,
    Limited,
    Refused,
}
