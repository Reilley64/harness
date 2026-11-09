use crate::child_runtime::ChildRuntime;
use crate::ports::ReadCoordination;
use crate::ports::UpdateCoordination;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) child_runtime: ChildRuntime,
    pub(crate) read_coordination: ReadCoordination,
    pub(crate) update_coordination: UpdateCoordination,
}
