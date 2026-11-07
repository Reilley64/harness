use crate::ModelSelection;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkApproval {
    pub intent: String,
    pub version: String,
    pub revision: u64,
    pub selection: ModelSelection,
    pub coordinator_id: String,
    pub approved_at: u64,
    pub delivered: bool,
}
