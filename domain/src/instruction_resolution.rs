use crate::Instruction;
use crate::InstructionSnapshot;

#[derive(Debug, Clone)]
pub struct InstructionResolution {
    pub available: Option<InstructionSnapshot>,
    pub blocked: Vec<String>,
    pub overrides: Vec<String>,
    pub bundled: Vec<Instruction>,
}
