#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkProposal {
    pub root: String,
    pub source_revision: String,
    pub spec: String,
    pub scope: String,
    pub criteria: String,
    pub dependencies: String,
    pub wiki: Option<String>,
    pub context: Option<String>,
    pub artifacts: Vec<String>,
    pub document: String,
    pub version: String,
}
impl WorkProposal {
    pub fn validate(&self) -> Result<(), &'static str> {
        for value in [
            &self.root,
            &self.source_revision,
            &self.spec,
            &self.scope,
            &self.criteria,
            &self.dependencies,
            &self.document,
        ] {
            if value.trim().is_empty() || value.len() > 262_144 || value.contains('\0') {
                return Err("Proposal fields must be nonempty, bounded UTF-8 text");
            }
        }
        if self.version.len() != 64
            || !self
                .version
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || self.artifacts.len() > 32
            || self
                .artifacts
                .iter()
                .chain(self.wiki.iter())
                .chain(self.context.iter())
                .any(|v| v.trim().is_empty() || v.len() > 4096 || v.contains('\0'))
        {
            return Err("Invalid proposal identity or context pointers");
        }
        Ok(())
    }
}
