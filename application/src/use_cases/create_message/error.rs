#[ohno::error]
#[display("failed to create message")]
pub struct Error;

#[ohno::error]
#[display("Message failed with {} retained tool causes", tool_errors.len())]
struct Retained {
    tool_errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

pub(super) fn retain(
    original: Error,
    tool_errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
) -> Error {
    if tool_errors.is_empty() {
        original
    } else {
        Error::caused_by(Retained::caused_by(tool_errors, original))
    }
}
