#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadTextMatchesArguments {
    pub pattern: String,
    pub path: Option<String>,
    pub glob: Option<String>,
    pub literal: Option<bool>,
    pub ignore_case: Option<bool>,
    pub context_lines: Option<usize>,
    pub limit: Option<usize>,
}
