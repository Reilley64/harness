#[ohno::error]
#[display("failed to drain owned Research executions")]
pub struct Error;

#[ohno::error]
#[display("Research failed with {} retained causes", diagnostics.len())]
struct Retained {
    diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

pub(super) fn retain(
    original: Error,
    diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
) -> Error {
    if diagnostics.is_empty() {
        original
    } else {
        Error::caused_by(Retained::caused_by(diagnostics, original))
    }
}
