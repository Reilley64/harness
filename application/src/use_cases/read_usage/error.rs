use crate::error_context::ErrorContext;

#[ohno::error]
#[display("failed to read deduplicated usage")]
pub struct Error;

impl Error {
    pub(super) fn blocked(message: &'static str) -> Self {
        Self::caused_by(ErrorContext::caused_by(message, message))
    }

    pub(super) fn unavailable(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::caused_by(ErrorContext::caused_by(
            "Usage source is unavailable, corrupt or changed; inspect retained storage and refresh.",
            error,
        ))
    }
}
