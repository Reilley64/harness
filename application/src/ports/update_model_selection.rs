use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update model selection")]
pub struct UpdateModelSelectionError;

port! {
    pub UpdateModelSelection(selected_model: domain::SelectedModel,) -> PortFuture<Result<domain::SelectedModel, UpdateModelSelectionError>>;
}
