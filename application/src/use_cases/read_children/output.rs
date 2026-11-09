#[derive(Debug)]
pub struct Output {
    pub coordination: Option<domain::Coordination>,
    pub session_revision: Option<u64>,

    pub current_work: Option<String>,
}
