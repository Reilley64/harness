use crate::ports;
#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_work: ports::ReadWork,
    pub(super) update_work: ports::UpdateWork,
    pub(super) read_work_proposal: ports::ReadWorkProposal,
}
