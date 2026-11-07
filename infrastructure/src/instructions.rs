use sha2::Digest;
use sha2::Sha256;

pub(crate) fn fingerprint(snapshot: &domain::InstructionSnapshot) -> String {
    let mut hash = Sha256::new();
    let mut add = |value: &str| {
        hash.update((value.len() as u64).to_le_bytes());
        hash.update(value.as_bytes());
    };
    add(&snapshot.phase);
    for values in [&snapshot.scopes, &snapshot.references] {
        add(&values.len().to_string());
        for value in values {
            add(value);
        }
    }
    add(&snapshot.instructions.len().to_string());
    for instruction in &snapshot.instructions {
        for value in [
            &instruction.id,
            &instruction.version,
            &instruction.source,
            &instruction.body,
        ] {
            add(value);
        }
        for values in [&instruction.upstream, &instruction.requires] {
            add(&values.len().to_string());
            for value in values {
                add(value);
            }
        }
    }
    format!("{:x}", hash.finalize())
}

pub(crate) fn validate(
    snapshot: &domain::InstructionSnapshot,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    snapshot.validate()?;
    if fingerprint(snapshot) != snapshot.id {
        return Err("Instruction snapshot content does not match its recorded identity".into());
    }
    Ok(())
}
