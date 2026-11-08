#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchAssignment {
    pub parent_id: String,
    pub intent: String,

    pub assignment_id: String,
    pub execution_id: String,
}
