#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadDirectoryArguments {
    pub path: Option<String>,
    pub limit: Option<usize>,
}
