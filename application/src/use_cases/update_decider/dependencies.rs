use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_models: ports::ReadModels,
    pub(super) update_decider: ports::UpdateDecider,
    pub(super) read_model_allowlist: ports::ReadModelAllowlist,
    pub(super) read_provider_connection: ports::ReadProviderConnection,
}
