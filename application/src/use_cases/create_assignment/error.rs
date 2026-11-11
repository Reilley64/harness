#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to create assignment")]
pub struct Error;
