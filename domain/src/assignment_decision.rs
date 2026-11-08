use crate::AssignmentBrief;
use crate::DecisionCandidate;
use crate::DecisionPhase;
use crate::DecisionRequest;
use crate::DecisionResponse;
use crate::ModelSelection;

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentDecision {
    pub id: String,
    pub parent_id: String,
    pub parent_agent_id: String,
    pub parent_revision: u64,
    pub parent: ModelSelection,
    pub parent_connection_id: String,
    pub work_id: String,
    pub work_intent: String,
    pub work_version: String,
    pub brief: AssignmentBrief,
    pub revision: u64,
    pub finished_at: Option<u64>,
    pub phase: DecisionPhase,
    pub request: Option<DecisionRequest>,
    pub response: Option<DecisionResponse>,
    pub selected: Option<DecisionCandidate>,
    pub reason: Option<String>,
}

impl AssignmentDecision {
    pub fn same_intent(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent_id == other.parent_id
            && self.parent_agent_id == other.parent_agent_id
            && self.work_id == other.work_id
            && self.work_intent == other.work_intent
            && self.work_version == other.work_version
            && self.brief == other.brief
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        self.brief.validate()?;
        if self.id != self.brief.intent
            || self.parent.model_id.is_empty()
            || self.parent_connection_id.is_empty()
            || self.work_version.len() != 64
            || self.reason.as_ref().is_some_and(|r| r.len() > 2048)
        {
            return Err("Invalid assignment decision");
        }
        if matches!(
            self.phase,
            DecisionPhase::Selected | DecisionPhase::Fallback
        ) && self.selected.is_none()
        {
            return Err("Selected decision is missing its exact pair");
        }
        if self.phase == DecisionPhase::Prepared
            && (self.request.is_some() || self.response.is_some() || self.selected.is_some())
            || self.phase == DecisionPhase::Pending
                && (self.request.is_none() || self.response.is_some() || self.selected.is_some())
        {
            return Err("Unsettled decision has inconsistent evidence");
        }
        if self.phase == DecisionPhase::Selected {
            let request = self
                .request
                .as_ref()
                .ok_or("Selected decision is missing its request")?;
            let response = self
                .response
                .as_ref()
                .ok_or("Selected decision is missing its response")?;
            request.validate_response(response)?;
            let chosen = response
                .choice
                .as_ref()
                .and_then(|key| request.candidates.iter().find(|c| &c.key == key));
            if chosen.is_none() || chosen != self.selected.as_ref() {
                return Err("Selected pair differs from the validated native choice");
            }
        }
        if let Some(request) = &self.request {
            request.validate_pool(self.brief.context_tokens)?;
        }
        if let Some(selected) = &self.selected {
            selected.validate(self.brief.context_tokens)?;
            if self.phase == DecisionPhase::Fallback
                && (selected.selection != self.parent
                    || selected.connection_id != self.parent_connection_id)
            {
                return Err("Fallback differs from the actual parent snapshot");
            }
        }
        Ok(())
    }

    pub fn validate_transition(&self, next: &Self) -> Result<(), &'static str> {
        if !self.same_intent(next)
            || self.parent != next.parent
            || self.parent_connection_id != next.parent_connection_id
            || self.parent_revision != next.parent_revision
            || self.revision != next.revision
            || self.phase != DecisionPhase::Prepared && self.phase != DecisionPhase::Pending
            || self.phase == DecisionPhase::Pending && self.request != next.request
            || next.phase == DecisionPhase::Prepared
        {
            return Err("Immutable or terminal decision changed");
        }
        next.validate()
    }
}
