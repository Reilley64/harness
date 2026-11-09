use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create Research conversation")]
pub struct CreateResearchError;

port! { pub CreateResearch(execution: domain::ChildExecution, workspace: std::path::PathBuf, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<String, CreateResearchError>>; }
