use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update assignment workspace")]
pub struct UpdateWorkspaceError;

port! {
    pub UpdateWorkspace(execution: domain::ChildExecution, repository: std::path::PathBuf, action: domain::WorkspaceAction, cancellation: tokio_util::sync::CancellationToken,) -> PortFuture<Result<domain::AssignmentWorkspace, UpdateWorkspaceError>>;
}
