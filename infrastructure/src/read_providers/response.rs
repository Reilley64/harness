#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Response {
    pub id: String,
    pub name: String,
}

impl From<Response> for domain::Provider {
    fn from(dto: Response) -> Self {
        Self {
            id: domain::ProviderId(dto.id),
            name: dto.name,
        }
    }
}
