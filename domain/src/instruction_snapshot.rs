use crate::Instruction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionSnapshot {
    pub id: String,
    pub phase: String,
    pub scopes: Vec<String>,
    pub references: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl InstructionSnapshot {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id.len() != 64
            || !self.id.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !matches!(
                self.phase.as_str(),
                "grilling" | "specification" | "tickets"
            )
            || self.instructions.is_empty()
            || self.instructions.len() > 64
            || self.scopes.len() > 32
            || self.references.len() > 32
        {
            return Err("Invalid instruction snapshot");
        }
        let mut ids = std::collections::HashSet::new();
        for instruction in &self.instructions {
            if instruction.id.is_empty()
                || instruction.id.len() > 1024
                || !ids.insert(&instruction.id)
                || instruction.version.is_empty()
                || instruction.version.len() > 256
                || instruction.source.is_empty()
                || instruction.source.len() > 4096
                || instruction.body.trim().is_empty()
                || instruction.body.len() > 262_144
                || instruction.body.contains('\0')
                || instruction.requires.len() > 32
                || instruction.upstream.len() > 32
                || instruction
                    .upstream
                    .iter()
                    .any(|source| source.len() > 4096)
            {
                return Err("Invalid required instruction");
            }
        }
        for instruction in &self.instructions {
            if instruction
                .requires
                .iter()
                .any(|required| !ids.contains(required))
            {
                return Err("Instruction snapshot is missing a required dependency");
            }
        }
        if self
            .instructions
            .iter()
            .map(|instruction| instruction.body.len())
            .sum::<usize>()
            > 1_048_576
        {
            return Err("Instruction snapshot exceeds 1 MiB");
        }
        if !ids.contains(&"role/griller".to_owned())
            || !ids.contains(&"contract".to_owned())
            || !ids.contains(&format!("procedure/{}", self.phase))
        {
            return Err("Instruction snapshot is missing its required role, contract or phase");
        }
        Ok(())
    }
}
