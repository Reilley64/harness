use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Configuration {
    pub integration: String,
    pub setup: Vec<String>,
    pub checks: Vec<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    pub format: u32,
    pub assignment_id: String,
    pub execution_id: String,
    pub parent_id: String,
    pub agent_id: String,
    pub work_id: String,
    pub work_version: String,

    pub repository: PathBuf,
    pub path: PathBuf,
    pub branch: String,
    pub marker: String,
    pub base: String,
    pub tip: String,
    pub configuration: Configuration,

    pub phase: String,
    pub submissions: Vec<SubmissionRecord>,
    pub cycle_allowance: u32,
    pub rework_runs: Vec<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SubmissionRecord {
    pub commit: String,
    pub results: Vec<CheckRecord>,
    pub complete: bool,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckRecord {
    pub command: String,
    pub status: String,
    pub output: String,
}

impl Record {
    pub fn projection(&self, artifact: String) -> domain::AssignmentWorkspace {
        domain::AssignmentWorkspace {
            assignment_id: self.assignment_id.clone(),
            path: self.path.clone(),
            branch: self.branch.clone(),
            base: self.base.clone(),
            tip: self.tip.clone(),
            submission: self
                .submissions
                .last()
                .filter(|s| s.complete)
                .map(|s| s.commit.clone()),
            checks: self.submissions.last().map_or_else(Vec::new, |submission| {
                submission
                    .results
                    .iter()
                    .map(|check| domain::WorkspaceCheck {
                        command: check.command.clone(),
                        status: domain::ToolStatus::parse(&check.status)
                            .unwrap_or(domain::ToolStatus::Unknown),
                        output: check.output.clone(),
                    })
                    .collect()
            }),
            artifact,
        }
    }
}
