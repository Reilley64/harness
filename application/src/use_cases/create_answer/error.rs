#[ohno::error]
#[display("Could not answer the exact child input request")]
pub struct Error;

#[ohno::error]
#[display("Child answer retained {} additional causes", diagnostics.len())]
struct Retained {
    diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

pub(super) fn retain(
    original: Error,
    diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
) -> Error {
    Error::caused_by(Retained::caused_by(diagnostics, original))
}
