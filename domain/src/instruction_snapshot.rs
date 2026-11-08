use crate::Instruction;
use std::collections::HashSet;

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
                "grilling" | "specification" | "tickets" | "coordination" | "setup"
            )
            || self.instructions.is_empty()
            || self.instructions.len() > 64
            || self.scopes.len() > 32
            || self.references.len() > 32
        {
            return Err("Invalid instruction snapshot");
        }
        let mut ids = HashSet::new();
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
        let mut done = HashSet::new();
        for instruction in &self.instructions {
            visit(
                &instruction.id,
                &self.instructions,
                &mut HashSet::new(),
                &mut done,
            )?;
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
        let role = match self.phase.as_str() {
            "coordination" => "role/chief-of-staff",
            "setup" => "role/setup",
            _ => "role/griller",
        };
        if !ids.contains(&role.to_owned())
            || !ids.contains(&"contract".to_owned())
            || !ids.contains(&format!("procedure/{}", self.phase))
        {
            return Err("Instruction snapshot is missing its required role, contract or phase");
        }
        Ok(())
    }
}

fn visit<'a>(
    id: &'a str,
    instructions: &'a [Instruction],
    active: &mut HashSet<&'a str>,
    done: &mut HashSet<&'a str>,
) -> Result<(), &'static str> {
    if done.contains(id) {
        return Ok(());
    }
    if !active.insert(id) {
        return Err("Required instruction dependencies contain a cycle");
    }
    let instruction = instructions
        .iter()
        .find(|instruction| instruction.id == id)
        .ok_or("Instruction snapshot is missing a required dependency")?;
    for required in &instruction.requires {
        visit(required, instructions, active, done)?;
    }
    active.remove(id);
    done.insert(id);
    Ok(())
}
