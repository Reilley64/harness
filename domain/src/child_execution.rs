use crate::ApprovedWork;
use crate::AssignmentBrief;
use crate::ChildExecutionStatus;
use crate::ChildOutcome;
use crate::DecisionCandidate;
use crate::InstructionSnapshot;
use crate::ModelSelection;

#[derive(Debug, Clone, PartialEq)]
pub struct ChildExecution {
    pub intent: String,
    pub assignment_id: String,
    pub agent_id: String,
    pub execution_id: String,

    pub parent_id: String,
    pub parent_agent_id: String,
    pub parent_run_id: String,
    pub parent_revision: u64,

    pub parent_selection: ModelSelection,
    pub parent_connection_id: String,

    pub work_id: Option<String>,
    pub work_intent: Option<String>,
    pub work_version: Option<String>,
    pub approved_at: Option<u64>,

    pub brief: AssignmentBrief,
    pub selected: DecisionCandidate,
    pub instructions: InstructionSnapshot,
    pub routed: bool,

    pub controller: String,
    pub status: ChildExecutionStatus,
    pub stop_requested: bool,

    pub outcome: Option<ChildOutcome>,

    pub delivered_to: Option<String>,
    pub follow_up_completed_by: Option<String>,
}

impl ChildExecution {
    pub fn authorize_work(&self, work: &ApprovedWork) -> Result<(), &'static str> {
        if self.work_id.as_deref() != Some(work.id.as_str()) {
            return Err("Research work differs from its accepted authority");
        }
        let approval = work.authorize(
            self.work_intent
                .as_deref()
                .ok_or("Research approval intent is missing")?,
            self.work_version
                .as_deref()
                .ok_or("Research work version is missing")?,
        )?;
        if approval.coordinator_id != self.parent_id
            || Some(approval.approved_at) != self.approved_at
        {
            return Err("Research approval differs from the exact accepted parent authority");
        }
        Ok(())
    }

    pub fn matches_request(&self, parent_id: &str, brief: &AssignmentBrief, routed: bool) -> bool {
        self.parent_id == parent_id
            && self.intent == brief.intent
            && self.brief == *brief
            && self.routed == routed
    }

    pub fn same_assignment(&self, other: &Self) -> bool {
        self.intent == other.intent
            && self.assignment_id == other.assignment_id
            && self.agent_id == other.agent_id
            && self.execution_id == other.execution_id
            && self.parent_id == other.parent_id
            && self.parent_agent_id == other.parent_agent_id
            && self.parent_run_id == other.parent_run_id
            && self.parent_revision == other.parent_revision
            && self.parent_selection == other.parent_selection
            && self.parent_connection_id == other.parent_connection_id
            && self.work_id == other.work_id
            && self.work_intent == other.work_intent
            && self.work_version == other.work_version
            && self.approved_at == other.approved_at
            && self.brief == other.brief
            && self.selected == other.selected
            && self.instructions == other.instructions
            && self.routed == other.routed
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        self.brief.validate()?;
        self.selected.validate(self.brief.context_tokens)?;
        self.instructions.validate()?;
        if self.brief.role != "research"
            || self.instructions.phase != "research"
            || self.intent != self.brief.intent
            || [
                &self.intent,
                &self.assignment_id,
                &self.agent_id,
                &self.execution_id,
                &self.parent_id,
                &self.parent_agent_id,
                &self.parent_run_id,
                &self.parent_connection_id,
                &self.controller,
            ]
            .iter()
            .any(|id| id.is_empty() || id.len() > 128)
            || self.routed != self.work_id.is_some()
            || self.work_id.is_some() != self.work_version.is_some()
            || self.work_id.is_some() != self.work_intent.is_some()
            || self.work_id.is_some() != self.approved_at.is_some()
            || self
                .work_id
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self
                .work_version
                .as_ref()
                .is_some_and(|version| version.len() != 64)
            || self.follow_up_completed_by.is_some() && self.delivered_to.is_none()
            || self
                .delivered_to
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
            || self
                .follow_up_completed_by
                .as_ref()
                .is_some_and(|id| id.is_empty() || id.len() > 128)
        {
            return Err("Research assignment facts are missing or inconsistent");
        }
        if !self.routed
            && (self.selected.selection != self.parent_selection
                || self.selected.connection_id != self.parent_connection_id)
        {
            return Err("Unrouted Research must retain its exact actual-parent pair");
        }
        match &self.outcome {
            Some(outcome) => {
                outcome.validate()?;
                if outcome.conversation_id != self.agent_id || outcome.status != self.status {
                    return Err("Child outcome differs from the admitted execution");
                }
            }
            None if self.delivered_to.is_some()
                || (self.status.terminal()
                    && !(self.status == ChildExecutionStatus::Unknown && self.stop_requested)) =>
            {
                return Err("Terminal execution or delivery is missing its durable receipt");
            }
            None => {}
        }
        Ok(())
    }
}
