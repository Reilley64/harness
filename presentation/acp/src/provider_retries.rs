use crate::agents;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/provider-retry/update", response = agents::Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request {
    pub session_id: String,
    pub revision: u64,
    pub id: String,
    pub additional: Option<u32>,
}

pub(crate) fn summary(retry: &domain::ProviderRetry) -> serde_json::Value {
    serde_json::json!({
        "replyId": retry.reply_id,
        "providerId": retry.failure.provider_id.0,
        "connectionId": retry.failure.connection_id,
        "modelId": retry.failure.model_id,
        "kind": match retry.failure.kind {
            domain::ProviderFailureKind::Transient => "transient",
            domain::ProviderFailureKind::Quota => "quota",
            domain::ProviderFailureKind::Permanent => "permanent",
        },
        "failures": retry.failures,
        "allowance": retry.allowance,
        "retryAt": retry.retry_at,
        "waiting": retry.waiting,
        "extensions": retry.extensions,
    })
}
