use crate::ChildExecutionStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildOutcome {
    pub id: String,
    pub status: ChildExecutionStatus,

    pub conversation_id: String,
    pub run_id: Option<String>,
    pub reply_id: Option<String>,

    pub summary: String,
    pub artifacts: Vec<String>,
}

impl ChildOutcome {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.status.terminal()
            || self.id.is_empty()
            || self.id.len() > 128
            || self.conversation_id.is_empty()
            || self.conversation_id.len() > 128
            || self
                .run_id
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self
                .reply_id
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self.summary.len() > 8192
            || self.artifacts.len() > 32
            || self
                .artifacts
                .iter()
                .any(|path| path.is_empty() || path.len() > 4096)
        {
            return Err("Child outcome is incomplete or exceeds its bounds");
        }
        Ok(())
    }
}
