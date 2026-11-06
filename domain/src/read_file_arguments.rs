#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadFileArguments {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}
