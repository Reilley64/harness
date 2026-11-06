#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcNotification, serde::Serialize, serde::Deserialize,
)]
#[notification(method = "_harness/authentication")]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationChallenge {
    pub authentication_id: Option<String>,
    pub provider_id: String,
    pub authorization_url: String,
}

pub fn error(
    error: application::use_cases::create_provider_connection::Error,
) -> agent_client_protocol::Error {
    let diagnostic = application::error_context::diagnostic(&error);
    agent_client_protocol::Error::new(-32002, "Could not create provider connection")
        .data(diagnostic)
}
