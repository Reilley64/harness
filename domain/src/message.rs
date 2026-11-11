use crate::GenerationAccounting;
use crate::MAX_MODEL_STEPS;
use crate::MessageRole;
use crate::MessageStatus;
use crate::ModelEffort;
use crate::ModelStep;
use crate::ProviderId;
use crate::SteeringInstruction;
use crate::TokenUsage;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub status: MessageStatus,
    pub model_id: String,
    pub provider_id: ProviderId,
    pub effort: Option<ModelEffort>,
    pub steps: Vec<ModelStep>,
    pub steering: Vec<SteeringInstruction>,
    pub generation_usage: Option<Vec<Option<TokenUsage>>>,
    pub generation_accounting: Option<Vec<GenerationAccounting>>,
}

impl Message {
    pub fn begin_generation(&mut self) -> Result<(), &'static str> {
        if self.role != MessageRole::Assistant || self.status != MessageStatus::InProgress {
            return Err("There is no reply in progress");
        }
        let usage = self
            .generation_usage
            .as_mut()
            .ok_or("Missing generation accounting")?;
        if usage.len() >= MAX_MODEL_STEPS {
            return Err("Turn generation budget exceeded");
        }
        let accounting = self
            .generation_accounting
            .get_or_insert_with(|| vec![GenerationAccounting::unknown(); usage.len()]);
        accounting.push(GenerationAccounting::unknown());
        usage.push(None);
        Ok(())
    }

    pub fn discard_generation_before_attempt(&mut self) -> Result<(), &'static str> {
        if self.role != MessageRole::Assistant || self.status != MessageStatus::InProgress {
            return Err("There is no reply in progress");
        }
        let reports = self
            .generation_usage
            .as_mut()
            .ok_or("Missing generation accounting")?;
        if reports.last() != Some(&None) || reports.len() <= self.steps.len() {
            return Err("There is no unattempted generation to discard");
        }
        reports.pop();
        if let Some(accounting) = &mut self.generation_accounting {
            accounting.pop();
        }
        Ok(())
    }

    pub fn report_generation(&mut self, usage: Option<TokenUsage>) -> Result<(), &'static str> {
        let report = self
            .generation_usage
            .as_mut()
            .and_then(|reports| reports.last_mut())
            .ok_or("There is no generation to account for")?;
        if report.is_some() {
            return Err("Generation usage was already reported");
        }
        *report = usage;
        Ok(())
    }
}
