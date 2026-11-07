use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) lock_conversation: ports::LockConversation,
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) update_conversation: ports::UpdateConversation,
    pub(super) read_instructions: ports::ReadInstructions,
}
