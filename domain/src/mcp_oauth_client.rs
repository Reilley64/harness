#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpOauthClient {
    Dynamic,
    Registered {
        client_id: String,
        redirect_uri: String,
        secret_reference: Option<String>,
    },
    MetadataDocument {
        client_id: String,
        redirect_uri: String,
    },
}
