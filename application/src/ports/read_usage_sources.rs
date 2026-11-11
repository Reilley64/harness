use crate::ports::PortFuture;

pub struct Response {
    pub conversations: Vec<String>,
    pub decisions: Vec<String>,
}

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to enumerate usage sources")]
pub struct ReadUsageSourcesError;

port! { pub ReadUsageSources(cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<Response, ReadUsageSourcesError>>; }
