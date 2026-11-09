use crate::ChildExecution;
use crate::ChildExecutionStatus;
use crate::JoinMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedJoin {
    pub id: String,
    pub run_id: String,

    pub mode: JoinMode,
    pub executions: Vec<String>,

    pub backgrounded: bool,
}

impl CapturedJoin {
    pub fn same_intent(&self, other: &Self) -> bool {
        self.id == other.id
            && self.run_id == other.run_id
            && self.mode == other.mode
            && self.executions == other.executions
    }

    pub fn ready(&self, children: &[ChildExecution]) -> Result<bool, &'static str> {
        let mut completed = 0;
        for id in &self.executions {
            let child = children
                .iter()
                .find(|child| &child.execution_id == id)
                .ok_or("Captured join is missing a required execution; inspect coordination")?;
            if child.status.terminal() {
                if child.status != ChildExecutionStatus::Completed {
                    return Ok(true);
                }
                completed += 1;
            }
        }
        Ok(self.backgrounded
            || match self.mode {
                JoinMode::Any => completed > 0,
                JoinMode::All => completed == self.executions.len(),
            })
    }
}
