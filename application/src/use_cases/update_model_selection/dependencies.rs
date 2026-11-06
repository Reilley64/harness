use crate::ports::ReadModels;
use crate::ports::UpdateModelSelection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) update_model_selection: UpdateModelSelection,
    pub(super) read_models: ReadModels,
}
