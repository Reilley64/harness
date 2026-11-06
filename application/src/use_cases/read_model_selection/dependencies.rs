use crate::ports::ReadModelSelection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_model_selection: ReadModelSelection,
}
