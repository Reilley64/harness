use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update model allowlist")]
pub struct UpdateModelAllowlistError;

#[derive(Debug, Clone)]
pub enum ModelAllowlistUpdate {
    Replace(domain::ModelAllowlist),
    InitializeProvider(domain::ProviderId),
}

port! {
    pub UpdateModelAllowlist(update: ModelAllowlistUpdate) -> PortFuture<Result<domain::ModelAllowlist, UpdateModelAllowlistError>>;
}
