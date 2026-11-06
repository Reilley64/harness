#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteeringInstruction {
    pub id: String,
    pub text: String,
    pub after_step: usize,
}
