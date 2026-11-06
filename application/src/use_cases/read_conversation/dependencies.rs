use crate::ports::LockConversation;
use crate::ports::ReadConversation;
use crate::ports::UpdateConversation;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_conversation: ReadConversation,
    pub(super) lock_conversation: LockConversation,
    pub(super) update_conversation: UpdateConversation,
}
