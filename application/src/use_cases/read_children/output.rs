#[derive(Debug)]
pub struct Output {
    pub coordination: Option<domain::Coordination>,

    pub current_work: Option<String>,
}
