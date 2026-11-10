use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_agent_index: ports::ReadAgentIndex,
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) read_coordination: ports::ReadCoordination,
}
