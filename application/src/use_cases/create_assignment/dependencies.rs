use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_work: ports::ReadWork,
    pub(super) read_decision: ports::ReadDecision,
    pub(super) update_decision: ports::UpdateDecision,
}
