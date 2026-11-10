use crate::ports::UpdateCoordination;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) update_coordination: UpdateCoordination,
}
