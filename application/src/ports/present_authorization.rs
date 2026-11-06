use crate::ports::CreateProviderConnectionError;

#[derive(Debug, Clone)]
pub struct AuthorizationChallenge {
    pub authorization_url: String,
}

port! {
    pub PresentAuthorization(
        challenge: AuthorizationChallenge,
    ) -> Result<(), CreateProviderConnectionError>;
}
