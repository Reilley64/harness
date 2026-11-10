pub struct Output {
    pub agents: Vec<(
        domain::Conversation,
        Option<domain::Coordination>,
        Option<domain::ChildExecution>,
    )>,
    pub next: Option<String>,
}
