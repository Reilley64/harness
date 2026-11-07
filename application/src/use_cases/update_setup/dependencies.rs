use crate::ports;
#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) lock_conversation: ports::LockConversation,
    pub(super) update_conversation: ports::UpdateConversation,
    pub(super) update_setup_draft: ports::UpdateSetupDraft,
    pub(super) create_tool_output: ports::CreateToolOutput,
}
