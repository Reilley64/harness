use crate::ports::LockConversation;
use crate::ports::ReadConversation;
use crate::ports::ReadWork;
use crate::ports::UpdateConversation;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) lock_conversation: LockConversation,
    pub(crate) read_conversation: ReadConversation,
    pub(crate) read_work: ReadWork,
    pub(crate) update_conversation: UpdateConversation,
}
