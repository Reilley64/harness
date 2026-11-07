use crate::ports::CreateConversation;
use crate::ports::CreateModelStepFactory;
use crate::ports::ReadModelAllowlist;
use crate::ports::ReadModels;
use crate::ports::ReadProviderConnection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_model_allowlist: ReadModelAllowlist,
    pub(super) create_model_step_factory: CreateModelStepFactory,
    pub(super) create_conversation: CreateConversation,
    pub(super) read_models: ReadModels,
    pub(super) read_provider_connection: ReadProviderConnection,
}
