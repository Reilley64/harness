use crate::ports::CreateProviderConnectionFactory;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) create_provider_connection_factory: CreateProviderConnectionFactory,
}
