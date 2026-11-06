#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadCodeMatchesArguments {
    pub pattern: String,
    pub language: String,
    pub path: Option<String>,
    pub glob: Option<String>,
    pub limit: Option<usize>,
}
