use crate::ChildOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildReceipt {
    pub outcome: ChildOutcome,
    pub delivered_to: Option<String>,
    pub follow_up_completed_by: Option<String>,
}

impl ChildReceipt {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.outcome.validate()?;
        if self.follow_up_completed_by.is_some() && self.delivered_to.is_none()
            || [&self.delivered_to, &self.follow_up_completed_by]
                .iter()
                .any(|id| {
                    id.as_ref()
                        .is_some_and(|id| id.is_empty() || id.len() > 128)
                })
        {
            return Err("Retained child receipt delivery or follow-up is invalid");
        }
        Ok(())
    }
}
