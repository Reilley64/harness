use crate::ports::PortFuture;
#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create coordinator")]
pub struct CreateCoordinatorError;
port! { pub CreateCoordinator(work: domain::ApprovedWork, approval: domain::WorkApproval, model: domain::Model, connection: domain::ProviderConnection, instructions: domain::InstructionSnapshot, adopt_default: bool) -> PortFuture<Result<String, CreateCoordinatorError>>; }
