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

        let root = matches!(self.path.as_str(), "AGENTS.md" | "CLAUDE.md");
        if root {
            if self.before.is_none() && self.roots.iter().take(2).any(|(_, body)| body.is_some()) {
                return Err("Modify the existing root instead of creating a duplicate");
            }
            if self
                .before
                .as_ref()
                .is_some_and(|base| !self.after.starts_with(base))
            {
                return Err(
                    "Setup root drafts must preserve all existing content and append only missing navigation",
                );
            }
        } else if self.path != ".harness/instructions/config.json" && self.before.is_some() {
            return Err(
                "Setup only fills missing Markdown guidance/wiki outputs; existing documents are preserved",
            );
        }
        Ok(())
    }

    pub fn validate_output(&self, wiki: Option<&str>) -> Result<(), &'static str> {
        self.validate()?;

        if matches!(
            self.path.as_str(),
            "AGENTS.md"
                | "CLAUDE.md"
                | ".harness/instructions/config.json"
                | "docs/agents/issue-tracker.md"
                | "docs/agents/domain.md"
                | "docs/agents/triage-labels.md"
                | "docs/agents/wiki.md"
        ) {
            return Ok(());
        }

        let wiki = wiki.ok_or(
            "Propose wiki guidance with its exact index pointer before proposing a new wiki index",
        )?;
        if !self.path.ends_with(".md")
            || (!wiki.contains(&format!("`{}`", self.path))
                && !wiki.contains(&format!("]({})", self.path)))
        {
            return Err(
                "Only a missing index explicitly referenced by wiki guidance is a Setup output",
            );
        }
        Ok(())
    }
}
