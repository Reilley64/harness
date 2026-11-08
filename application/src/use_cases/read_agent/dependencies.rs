use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_setup: ports::ReadSetup,
    pub(super) read_agent: ports::ReadAgent,
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) update_conversation: ports::UpdateConversation,
    pub(super) lock_conversation: ports::LockConversation,
}
