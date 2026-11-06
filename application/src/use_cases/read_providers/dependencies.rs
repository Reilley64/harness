use crate::ports::ReadProviders;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_providers: ReadProviders,
}
