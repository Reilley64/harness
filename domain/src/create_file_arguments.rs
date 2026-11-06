#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateFileArguments {
    pub path: String,
    pub content: String,
}
