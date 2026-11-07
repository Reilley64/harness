use crate::ports::ReadModelAllowlist;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_model_allowlist: ReadModelAllowlist,
}
