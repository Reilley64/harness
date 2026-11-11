#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildInstruction {
    pub id: String,
    pub execution_id: String,
    pub order: u64,
    pub text: String,

    pub applied_run: Option<String>,
    pub applied_reply: Option<String>,
}

impl ChildInstruction {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id.is_empty()
            || self.id.len() > 128
            || self.execution_id.is_empty()
            || self.execution_id.len() > 128
            || self.order == 0
            || self.text.trim().is_empty()
            || self.text.len() > 16_384
            || self.applied_run.is_some() != self.applied_reply.is_some()
            || self
                .applied_run
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self
                .applied_reply
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
        {
            return Err("Child instruction identity, scope, order or acknowledgement is invalid");
        }
        Ok(())
    }
}
