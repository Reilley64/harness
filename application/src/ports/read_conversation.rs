use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read conversation")]
pub struct ReadConversationError;

port! {
    pub ReadConversation(id: String,) -> PortFuture<Result<domain::Conversation, ReadConversationError>>;
}
