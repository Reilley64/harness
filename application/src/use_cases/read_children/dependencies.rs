use crate::ports::ReadConversation;
use crate::ports::ReadCoordination;
use crate::ports::ReadExecutionClaims;
use crate::ports::UpdateCoordination;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) read_coordination: ReadCoordination,
    pub(crate) read_conversation: ReadConversation,

    pub(crate) read_execution_claims: ReadExecutionClaims,
    pub(crate) update_coordination: UpdateCoordination,
}
