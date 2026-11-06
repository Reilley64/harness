use crate::ports::CreateProviderConnection;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to select provider connection adapter")]
pub struct CreateProviderConnectionFactoryError;

port! {
    pub CreateProviderConnectionFactory(
        provider_id: domain::ProviderId,
    ) -> Result<CreateProviderConnection, CreateProviderConnectionFactoryError>;
}
