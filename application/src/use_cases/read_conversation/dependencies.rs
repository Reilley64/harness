use crate::ports::LockConversation;
use crate::ports::LockExecution;
use crate::ports::ReadConversation;
use crate::ports::ReadCoordination;
use crate::ports::UpdateConversation;
use crate::ports::UpdateCoordination;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_coordination: ReadCoordination,
    pub(super) update_coordination: UpdateCoordination,
    pub(super) read_conversation: ReadConversation,
    pub(super) lock_conversation: LockConversation,
    pub(super) lock_execution: LockExecution,
    pub(super) update_conversation: UpdateConversation,
}
