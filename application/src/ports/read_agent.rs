use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read agent")]
pub struct ReadAgentError;

port! {
    pub ReadAgent(workspace: std::path::PathBuf, role: domain::AgentRole,) -> PortFuture<Result<Option<String>, ReadAgentError>>;
}
