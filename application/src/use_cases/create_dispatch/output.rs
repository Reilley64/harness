#[derive(Debug)]
pub struct Output {
    pub diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,

    pub execution: domain::ChildExecution,

    pub accepted: bool,
}
