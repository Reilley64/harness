use crate::AgentId;
use crate::AgentRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub id: AgentId,
    pub role: AgentRole,
}
