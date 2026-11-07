use std::path::Component;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupDraft {
    pub workspace: std::path::PathBuf,
    pub path: String,
    pub before: Option<String>,
    pub after: String,
    pub roots: Vec<(String, Option<String>)>,
}

impl SetupDraft {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.workspace.is_absolute()
            || self.path.is_empty()
            || self.path.len() > 4096
            || std::path::Path::new(&self.path)
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            || self
                .before
                .as_ref()
                .is_some_and(|text| text.len() > 1_048_576)
            || self.after.len() > 1_048_576
            || self.roots.len() != 3
            || self.roots[0].0 != "AGENTS.md"
            || self.roots[1].0 != "CLAUDE.md"
            || self.roots[2].0 != ".harness/instructions/config.json"
            || self
                .roots
                .iter()
                .any(|(_, text)| text.as_ref().is_some_and(|text| text.len() > 1_048_576))
        {
            return Err("Invalid Setup draft");
        }
        Ok(())
    }
}
