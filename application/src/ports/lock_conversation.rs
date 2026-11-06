use crate::ports::PortFuture;

pub struct ConversationGuard(pub Box<dyn Send + Sync>);

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to lock conversation")]
pub struct LockConversationError;

port! {
    pub LockConversation(id: String,) -> PortFuture<Result<ConversationGuard, LockConversationError>>;
}
