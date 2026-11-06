use crate::TextEdit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateFileArguments {
    pub path: String,
    pub edits: Vec<TextEdit>,
}
