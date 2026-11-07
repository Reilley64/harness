use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) read_instructions: ports::ReadInstructions,
}
