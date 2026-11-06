#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcRequest, serde::Serialize, serde::Deserialize,
)]
#[request(method="_harness/conversation/steer",response=Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request {
    pub session_id: String,
    pub id: String,
    pub text: String,
}
#[derive(
    Debug, Clone, agent_client_protocol::JsonRpcResponse, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct Response {
    pub accepted: bool,
}

pub(crate) fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(b)
            }
        })
}
