#[derive(Debug)]
pub struct Output {
    pub id: String,
    pub provider_id: domain::ProviderId,
}

impl From<domain::ProviderConnection> for Output {
    fn from(connection: domain::ProviderConnection) -> Self {
        Self {
            id: connection.id,
            provider_id: connection.provider_id,
        }
    }
}
