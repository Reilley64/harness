#[ohno::error]
#[display(
    "Could not approve the exact execution scope version; inspect retained approval before retry"
)]
pub struct Error;

#[ohno::error]
#[display("Scope continuation retained {} additional causes", diagnostics.len())]
struct Retained {
    diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

pub(super) fn retain(mut diagnostics: Vec<Box<dyn std::error::Error + Send + Sync>>) -> Error {
    let original = diagnostics.remove(0);
    Error::caused_by(Retained::caused_by(diagnostics, original))
}
