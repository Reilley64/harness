use crate::ports::CreateConversation;
use crate::ports::ReadModels;
use crate::ports::ReadProviderConnection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) create_conversation: CreateConversation,
    pub(super) read_models: ReadModels,
    pub(super) read_provider_connection: ReadProviderConnection,
}
