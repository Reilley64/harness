use crate::WorkProposal;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkAssignment {
    pub work_id: String,
    pub intent: String,
    pub creator_id: String,
    pub proposal: WorkProposal,
    pub approved_at: u64,
}
