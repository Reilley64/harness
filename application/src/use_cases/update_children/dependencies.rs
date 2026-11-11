use crate::ports::ReadConversation;
use crate::ports::ReadCoordination;
use crate::ports::UpdateCoordination;
use crate::use_cases::delete_execution;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) delete_execution: delete_execution::Dependencies,

    pub(crate) read_conversation: ReadConversation,
    pub(crate) read_coordination: ReadCoordination,
    pub(crate) update_coordination: UpdateCoordination,
}
