use crate::McpOauthClient;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpOauth {
    pub issuer: String,
    pub client: McpOauthClient,
}
