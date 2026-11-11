use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create agent")]
pub struct CreateAgentError;

port! {
    pub CreateAgent(model: domain::Model, effort: Option<domain::ModelEffort>, connection: domain::ProviderConnection, workspace: std::path::PathBuf, role: domain::AgentRole,) -> PortFuture<Result<String, CreateAgentError>>;
}
