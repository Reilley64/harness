use crate::SetupDraft;
use crate::Tool;
use crate::ToolResult;
use crate::ToolStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub draft: Option<Box<SetupDraft>>,
    pub id: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    pub status: ToolStatus,
    pub output: String,
    pub artifact: Option<String>,
    pub truncated: bool,
    pub elapsed_ms: u64,
    pub text_offset: usize,
}

impl ToolCall {
    pub fn start(&mut self) -> Result<(), &'static str> {
        if !self.permits(ToolStatus::Running) {
            return Err("Only requested tools can start execution");
        }
        self.status = ToolStatus::Running;
        Ok(())
    }

    pub fn finish(&mut self, result: ToolResult) -> Result<(), &'static str> {
        if !result.status.terminal() || !self.permits(result.status) {
            return Err("Invalid tool completion transition");
        }
        self.status = result.status;
        self.output = result.output;
        self.artifact = result.artifact;
        self.truncated = result.truncated;
        Ok(())
    }

    pub(crate) fn permits(&self, status: ToolStatus) -> bool {
        match self.status {
            ToolStatus::Requested => matches!(
                status,
                ToolStatus::Running | ToolStatus::Invalid | ToolStatus::Skipped
            ),
            ToolStatus::Draft => matches!(status, ToolStatus::Running | ToolStatus::Skipped),
            ToolStatus::Running => status.terminal(),
            _ => false,
        }
    }

    pub fn stop(&mut self) {
        if !self.status.terminal() {
            self.status = if self.status == ToolStatus::Running {
                ToolStatus::Unknown
            } else {
                ToolStatus::Skipped
            };
            self.output = "The turn stopped before a confirmed result. Inspect unknown effects before retrying.".into();
        }
    }

    pub fn recover(&mut self) -> bool {
        if self.status.terminal() {
            return false;
        }
        self.status = if self.status == ToolStatus::Running
            && Tool::parse(&self.name).is_none_or(Tool::may_mutate)
        {
            ToolStatus::Unknown
        } else {
            ToolStatus::Interrupted
        };
        self.output =
            "Execution was interrupted; no automatic retry. Effects may already have occurred."
                .into();
        true
    }
}
