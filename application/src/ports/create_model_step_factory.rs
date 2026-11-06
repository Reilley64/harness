use crate::ports::CreateModelStep;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to select reply adapter")]
pub struct CreateModelStepFactoryError;

port! {
    pub CreateModelStepFactory(provider_id: domain::ProviderId,) -> Result<CreateModelStep, CreateModelStepFactoryError>;
}
