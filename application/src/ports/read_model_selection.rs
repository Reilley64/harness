use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read model selection")]
pub struct ReadModelSelectionError;

port! {
    pub ReadModelSelection() -> PortFuture<Result<Option<domain::SelectedModel>, ReadModelSelectionError>>;
}
