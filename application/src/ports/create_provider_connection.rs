use crate::ports::PortFuture;
use crate::ports::PresentAuthorization;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create provider connection")]
pub struct CreateProviderConnectionError;

port! {
    pub CreateProviderConnection(
        present_authorization: PresentAuthorization,
    ) -> PortFuture<Result<domain::ProviderConnection, CreateProviderConnectionError>>;
}
