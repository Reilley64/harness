#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadWebSearchArguments {
    pub query: String,
    pub limit: Option<usize>,
}
