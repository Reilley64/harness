use crate::ports::PortFuture;

pub struct Response {
    pub workspace: std::path::PathBuf,
    pub conversations: Vec<String>,
    pub next: Option<String>,
}

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read the agent index")]
pub struct ReadAgentIndexError;

port! {
    pub ReadAgentIndex(workspace: std::path::PathBuf, after: Option<String>, cancellation_token: tokio_util::sync::CancellationToken) -> PortFuture<Result<Response, ReadAgentIndexError>>;
}
