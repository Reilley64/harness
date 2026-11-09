use crate::ports::ReadConversation;
use crate::ports::ReadCoordination;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) read_coordination: ReadCoordination,
    pub(crate) read_conversation: ReadConversation,
}
