use crate::ports::CreateModelStepFactory;
use crate::ports::ReadModelAllowlist;
use crate::ports::ReadModels;
use crate::ports::ReadProviderConnection;
use crate::ports::UpdateModelSelection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_model_allowlist: ReadModelAllowlist,
    pub(super) read_provider_connection: ReadProviderConnection,
    pub(super) create_model_step_factory: CreateModelStepFactory,
    pub(super) update_model_selection: UpdateModelSelection,
    pub(super) read_models: ReadModels,
}
