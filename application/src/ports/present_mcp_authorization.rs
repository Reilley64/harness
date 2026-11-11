use crate::ports::UpdateMcpConfigurationError;

port! {
    pub PresentMcpAuthorization(authorization_url: String) -> Result<(), UpdateMcpConfigurationError>;
}
