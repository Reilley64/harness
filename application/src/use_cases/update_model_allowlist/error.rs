#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("Could not update model allowlist; open /model settings")]
pub struct Error;
