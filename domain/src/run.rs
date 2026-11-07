use crate::AgentId;
use crate::ModelEffort;
use crate::ProviderId;
use crate::RunStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub id: String,
    pub agent_id: AgentId,
    pub reply_id: String,
    pub model_id: String,
    pub provider_id: ProviderId,
    pub effort: Option<ModelEffort>,
    pub status: RunStatus,
    pub instructions: Option<String>,
}
