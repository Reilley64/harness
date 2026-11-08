use super::Error;

pub struct Output {
    pub decision: domain::AssignmentDecision,
    pub blocked: Option<String>,
    pub diagnostics: Vec<Error>,
}
