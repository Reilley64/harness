use crate::ports;
#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_work: ports::ReadWork,
}
