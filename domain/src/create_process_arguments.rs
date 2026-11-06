#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProcessArguments {
    pub command: String,
    pub timeout_seconds: Option<u64>,
}
