#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentBrief {
    pub intent: String,
    pub role: String,
    pub task: String,
    pub constraints: String,
    pub tool_context: String,
    pub context_tokens: u64,
}

impl AssignmentBrief {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.role.as_str(), "research" | "implementer" | "reviewer")
            || self.task.is_empty()
            || self.task.len() > 4096
            || self.constraints.len() > 2048
            || self.tool_context.len() > 2048
            || self.context_tokens == 0
            || self.context_tokens > 1_000_000_000
        {
            return Err("Assignment brief exceeds supported bounds");
        }
        Ok(())
    }
}
