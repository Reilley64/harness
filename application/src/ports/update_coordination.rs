use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update coordination")]
pub struct UpdateCoordinationError {
    pub conflict: bool,
}

impl UpdateCoordinationError {
    pub fn with_conflict(mut self, conflict: bool) -> Self {
        self.conflict = conflict;
        self
    }
}

#[derive(Debug)]
pub struct Response {
    pub coordination: domain::Coordination,

    pub changed: bool,
    pub claimed: bool,
}

port! { pub UpdateCoordination(agent_id: String, command: domain::CoordinationCommand, revision: Option<u64>, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<Response, UpdateCoordinationError>>; }
