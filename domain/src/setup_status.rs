use crate::AgentRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStatus {
    pub workspace: std::path::PathBuf,
    pub roots: Vec<String>,
    pub missing: Vec<String>,
    pub triage: bool,
}

impl SetupStatus {
    pub fn role(&self) -> AgentRole {
        if self.missing.is_empty() {
            AgentRole::Griller
        } else {
            AgentRole::Setup
        }
    }
}
