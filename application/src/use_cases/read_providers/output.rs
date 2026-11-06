#[derive(Debug)]
pub struct Output {
    pub id: domain::ProviderId,
    pub name: String,
}

impl From<domain::Provider> for Output {
    fn from(provider: domain::Provider) -> Self {
        Self {
            id: provider.id,
            name: provider.name,
        }
    }
}
