use crate::ports::CreateProviderConnectionFactory;
use crate::ports::UpdateModelAllowlist;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) update_model_allowlist: UpdateModelAllowlist,
    pub(super) create_provider_connection_factory: CreateProviderConnectionFactory,
}
