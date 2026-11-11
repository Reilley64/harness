use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_decision: ports::ReadDecision,
    pub(super) read_models: ports::ReadModels,
    pub(super) read_model_allowlist: ports::ReadModelAllowlist,
    pub(super) read_provider_connection: ports::ReadProviderConnection,
    pub(super) create_model_step_factory: ports::CreateModelStepFactory,
}
