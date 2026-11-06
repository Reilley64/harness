#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadPathsArguments {
    pub pattern: String,
    pub path: Option<String>,
    pub limit: Option<usize>,
}
