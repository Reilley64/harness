#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcNotification, serde::Serialize, serde::Deserialize,
)]
#[notification(method = "_harness/usage")]
#[serde(rename_all = "camelCase")]
pub(crate) struct Notification {
    pub session_id: String,
    pub usage: Summary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Summary {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub context_tokens: Option<u64>,
    pub model_id: Option<String>,
    pub partial: bool,
}
impl From<domain::UsageSummary> for Summary {
    fn from(value: domain::UsageSummary) -> Self {
        Self {
            input_tokens: value.input_tokens,
            output_tokens: value.output_tokens,
            context_tokens: value.context_tokens,
            model_id: value.model_id,
            partial: value.partial,
        }
    }
}
