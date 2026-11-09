#[ohno::error]
#[display(
    "Could not approve the exact execution scope version; inspect retained approval before retry"
)]
pub struct Error;
