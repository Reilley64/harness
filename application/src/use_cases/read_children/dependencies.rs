use crate::ports::ReadCoordination;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) read_coordination: ReadCoordination,
}
