use crate::AgentRole;
use crate::Conversation;
use crate::InstructionSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchGuidance {
    pub intent: String,
    pub parent_id: String,
    pub parent_agent_id: String,

    pub work_id: Option<String>,
    pub work_intent: Option<String>,
    pub work_version: Option<String>,

    pub snapshot: InstructionSnapshot,
}

impl ResearchGuidance {
    pub fn validate(&self) -> Result<(), &'static str> {
        if [&self.intent, &self.parent_id, &self.parent_agent_id]
            .into_iter()
            .any(|id| id.is_empty() || id.len() > 128)
            || self.work_id.is_some() != self.work_intent.is_some()
            || self.work_id.is_some() != self.work_version.is_some()
            || !matches!(self.snapshot.phase.as_str(), "research" | "implementation")
            || self.snapshot.phase == "implementation" && self.work_id.is_none()
        {
            return Err("Research guidance must bind the exact parent, intent and approved work");
        }
        self.snapshot.validate()
    }

    pub fn matches(&self, parent: &Conversation) -> bool {
        (self.snapshot.phase != "implementation"
            || parent
                .agent
                .as_ref()
                .is_some_and(|agent| agent.role == AgentRole::ChiefOfStaff))
            && self.parent_id == parent.id
            && parent
                .agent
                .as_ref()
                .is_some_and(|agent| agent.id.0 == self.parent_agent_id)
            && self.work_id.as_deref()
                == parent.assignment.as_ref().map(|work| work.work_id.as_str())
            && self.work_intent.as_deref()
                == parent.assignment.as_ref().map(|work| work.intent.as_str())
            && self.work_version.as_deref()
                == parent
                    .assignment
                    .as_ref()
                    .map(|work| work.proposal.version.as_str())
    }
}
