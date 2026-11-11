use crate::error_context::ErrorContext;

#[ohno::error]
#[display("failed to update provider retry authority")]
pub struct Error;

impl Error {
    pub(super) fn blocked(message: &'static str) -> Self {
        Self::caused_by(ErrorContext::caused_by(message, message))
    }

    pub(super) fn unavailable(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::caused_by(ErrorContext::caused_by(
            "Retry authority could not be updated; inspect the current wait, approved work and storage before retrying.",
            error,
        ))
    }
}
