use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) create_agent: ports::CreateAgent,
    pub(super) read_agent: ports::ReadAgent,
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) update_conversation: ports::UpdateConversation,
    pub(super) lock_conversation: ports::LockConversation,
    pub(super) read_models: ports::ReadModels,
    pub(super) read_provider_connection: ports::ReadProviderConnection,
    pub(super) read_model_allowlist: ports::ReadModelAllowlist,
    pub(super) create_model_step_factory: ports::CreateModelStepFactory,
}
