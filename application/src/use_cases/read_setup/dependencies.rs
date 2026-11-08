use crate::ports;
#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_setup: ports::ReadSetup,
    pub(super) read_conversation: ports::ReadConversation,
}
