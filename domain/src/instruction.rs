#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub id: String,
    pub version: String,
    pub source: String,
    pub upstream: Vec<String>,
    pub requires: Vec<String>,
    pub body: String,
}
