use crate::WorkProposal;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPublication {
    pub intent: String,
    pub proposal: WorkProposal,
}
