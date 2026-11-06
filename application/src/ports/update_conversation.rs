use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update conversation")]
pub struct UpdateConversationError;

port! {
    pub UpdateConversation(conversation: domain::Conversation,) -> PortFuture<Result<domain::Conversation, UpdateConversationError>>;
}
