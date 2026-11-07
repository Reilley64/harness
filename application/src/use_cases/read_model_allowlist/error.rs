#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("Could not read model allowlist; open /model settings")]
pub struct Error;
