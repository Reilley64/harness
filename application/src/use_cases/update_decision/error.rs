#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to select native decision")]
pub struct Error;
