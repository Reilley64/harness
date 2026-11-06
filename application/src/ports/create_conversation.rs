use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create conversation")]
pub struct CreateConversationError;

port! {
    pub CreateConversation(model: domain::Model, effort: Option<domain::ModelEffort>, connection: domain::ProviderConnection, workspace: std::path::PathBuf,) -> PortFuture<Result<domain::Conversation, CreateConversationError>>;
}
