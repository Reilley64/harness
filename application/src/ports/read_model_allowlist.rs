use crate::ports::PortFuture;

#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read model allowlist")]
pub struct ReadModelAllowlistError;

port! {
    pub ReadModelAllowlist() -> PortFuture<Result<domain::ModelAllowlist, ReadModelAllowlistError>>;
}
