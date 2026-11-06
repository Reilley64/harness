use crate::ports::ReadModels;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_models: ReadModels,
}
