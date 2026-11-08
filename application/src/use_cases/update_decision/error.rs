#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("failed to select native decision")]
pub struct Error {
    pub diagnostics: Vec<Error>,
}

impl Error {
    pub(crate) fn with_cause(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::caused_by(Vec::new(), error)
    }
}
