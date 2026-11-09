#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeAmendment {
    pub id: String,
    pub execution_id: String,
    pub version: String,
    pub text: String,

    pub replaces: Option<String>,
    pub supersedes_run: Option<String>,
    pub approved_by: Option<String>,
}

impl ScopeAmendment {
    pub fn validate(&self) -> Result<(), &'static str> {
        if [&self.id, &self.execution_id, &self.version]
            .iter()
            .any(|id| id.is_empty() || id.len() > 128)
            || [&self.replaces, &self.supersedes_run, &self.approved_by]
                .iter()
                .any(|id| {
                    id.as_ref()
                        .is_some_and(|id| id.is_empty() || id.len() > 128)
                })
            || self.text.trim().is_empty()
            || self.text.len() > 8192
        {
            return Err("Scope amendment identity, exact version or criteria is invalid");
        }
        Ok(())
    }
}
