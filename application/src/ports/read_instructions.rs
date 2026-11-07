use crate::ports::PortFuture;

#[ohno::error]
#[from(Box<dyn std::error::Error + Send + Sync>)]
#[display("Could not resolve required instructions")]
pub struct ReadInstructionsError;

port! { pub ReadInstructions(workspace: std::path::PathBuf, phase: String, scopes: Vec<String>, references: Vec<String>,) -> PortFuture<Result<domain::InstructionResolution, ReadInstructionsError>>; }
