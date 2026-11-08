#[derive(Debug)]
pub struct Output {
    pub conversation: domain::Conversation,
    pub tool_errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
}
