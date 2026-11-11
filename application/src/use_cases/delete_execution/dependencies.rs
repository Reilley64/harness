use crate::child_runtime::ChildRuntime;
use crate::use_cases::read_children;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) child_runtime: ChildRuntime,
    pub(crate) read_children: read_children::Dependencies,
}
