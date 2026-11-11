use crate::ports::ReadConversation;
use crate::ports::ReadDecision;
use crate::ports::ReadUsageSources;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(crate) read_usage_sources: ReadUsageSources,
    pub(crate) read_conversation: ReadConversation,
    pub(crate) read_decision: ReadDecision,
}
