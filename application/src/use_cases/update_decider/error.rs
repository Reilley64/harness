#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to update decider")]
pub struct Error;
