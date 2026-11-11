#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to read deciders")]
pub struct Error;
