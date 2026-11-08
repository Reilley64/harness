#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Draft,
    Requested,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Invalid,
    Skipped,
    Interrupted,
    Unknown,
}

impl ToolStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Requested => "requested",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Invalid => "invalid",
            Self::Skipped => "skipped",
            Self::Interrupted => "interrupted",
            Self::Unknown => "unknown",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        [
            Self::Draft,
            Self::Requested,
            Self::Running,
            Self::Succeeded,
            Self::Failed,
            Self::Cancelled,
            Self::Invalid,
            Self::Skipped,
            Self::Interrupted,
            Self::Unknown,
        ]
        .into_iter()
        .find(|status| status.as_str() == value)
    }
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Requested | Self::Running)
    }
}
