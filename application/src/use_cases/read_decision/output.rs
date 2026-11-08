use super::Error;

pub struct Output {
    pub diagnostics: Vec<Error>,
    pub decision: Option<domain::AssignmentDecision>,
    pub blocked: Option<String>,
}
