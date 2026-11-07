use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update work")]
pub struct UpdateWorkError;
port! { pub UpdateWork(work: domain::ApprovedWork, revision: Option<u64>) -> PortFuture<Result<domain::ApprovedWork, UpdateWorkError>>; }
