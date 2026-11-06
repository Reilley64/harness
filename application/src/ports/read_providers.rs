use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read providers")]
pub struct ReadProvidersError;

port! {
    pub ReadProviders() -> PortFuture<Result<Vec<domain::Provider>, ReadProvidersError>>;
}
