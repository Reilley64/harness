#[ohno::error]
#[derive(Default)]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("MCP configuration request failed")]
pub struct Error;
