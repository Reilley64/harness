use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read models")]
pub struct ReadModelsError;

port! {
    pub ReadModels() -> PortFuture<Result<Vec<domain::Model>, ReadModelsError>>;
}
