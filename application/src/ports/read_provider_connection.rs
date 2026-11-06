use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read provider connection")]
pub struct ReadProviderConnectionError;

port! {
    pub ReadProviderConnection(provider_id: domain::ProviderId,) -> PortFuture<Result<domain::ProviderConnection, ReadProviderConnectionError>>;
}
