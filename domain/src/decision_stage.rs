use crate::DecisionRequest;
use crate::DecisionResponse;

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionStage {
    pub id: String,
    pub level: u32,
    pub members: Vec<String>,
    pub input: String,

    pub started_at: Option<u64>,
    pub response: Option<DecisionResponse>,
    pub failure: Option<String>,
    pub finished_at: Option<u64>,
}

impl DecisionStage {
    pub fn validate(&self, request: &DecisionRequest) -> Result<(), &'static str> {
        if self.id.is_empty()
            || self.members.is_empty()
            || self.members.len() > 254
            || self.input.len() > 24_576
            || self.input.len() as u64 > request.model.context_window.unwrap_or(0)
            || self.failure.as_ref().is_some_and(|f| f.len() > 2048)
        {
            return Err("Invalid bounded native stage");
        }
        if self.started_at.is_none() && (self.finished_at.is_some() || self.response.is_some()) {
            return Err("Unadmitted native stage cannot have a completion or report");
        }
        if (self.finished_at.is_none() || self.failure.is_some())
            && self.response.as_ref().is_some_and(|r| {
                r.choice.is_some() || r.confidence.is_some() || !r.probabilities.is_empty()
            })
        {
            return Err("Unsettled or failed native stage retains only safe accounting");
        }
        let mut seen = std::collections::HashSet::new();
        for key in &self.members {
            if !seen.insert(key)
                || !request
                    .candidates
                    .iter()
                    .any(|candidate| &candidate.key == key)
            {
                return Err("Native stage membership differs from the complete snapshot");
            }
        }
        if let Some(response) = &self.response {
            response.validate_report()?;
            if self.finished_at.is_some() && self.failure.is_none() {
                let mut keys: Vec<_> = self.members.iter().map(String::as_str).collect();
                keys.push("abstain");
                response.validate(&keys)?;
                request.validate_identity(response)?;
                match response.choice.as_deref() {
                    None => return Err("Native refusal: required stage has no winner"),
                    Some("abstain") => {
                        return Err("Explicit native abstention: required stage has no winner");
                    }
                    _ => {}
                }
            }
        }
        if self.finished_at.is_some() && self.failure.is_none() && self.response.is_none() {
            return Err("Successful stage is missing its response");
        }
        Ok(())
    }

    pub fn status(&self) -> &'static str {
        match (self.started_at, self.finished_at, self.failure.is_some()) {
            (None, None, false) => "planned",
            (None, None, true) => "stopped_before_admission",
            (Some(_), None, false) => "pending",
            (Some(_), None, true) => "stopped_unknown",
            (Some(_), Some(_), false) => "completed",
            (Some(_), Some(_), true) => "failed",
            _ => "invalid",
        }
    }

    pub fn same_input(&self, other: &Self) -> bool {
        self.id == other.id
            && self.level == other.level
            && self.members == other.members
            && self.input == other.input
    }
}
