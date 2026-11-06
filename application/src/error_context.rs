#[ohno::error]
#[display("{message}")]
pub struct ErrorContext {
    pub message: &'static str,
}

pub trait ResultExt<T> {
    fn context(self, message: &'static str) -> Result<T, ErrorContext>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fn context(self, message: &'static str) -> Result<T, ErrorContext> {
        self.map_err(|error| ErrorContext::caused_by(message, error))
    }
}

pub fn diagnostic(error: &(dyn std::error::Error + 'static)) -> String {
    let mut cause = error;
    loop {
        if let Some(context) = cause.downcast_ref::<ErrorContext>() {
            return context.message.to_owned();
        }
        match cause.source() {
            Some(source) => cause = source,
            None => {
                return "The operation failed. Check your configuration and try again.".to_owned();
            }
        }
    }
}
