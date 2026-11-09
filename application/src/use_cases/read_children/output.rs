#[derive(Debug)]
pub struct Output {
    pub diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,

    pub coordination: Option<domain::Coordination>,
    pub session_revision: Option<u64>,

    pub current_work: Option<String>,
}
