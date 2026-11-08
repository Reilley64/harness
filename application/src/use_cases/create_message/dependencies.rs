use crate::ports;
use crate::ports::ReadModelAllowlist;
use crate::ports::ReadProviderConnection;
use crate::use_cases::create_work;

use crate::ports::CreateModelStepFactory;
use crate::ports::LockConversation;
use crate::ports::ReadConversation;
use crate::ports::ReadModels;
use crate::ports::UpdateConversation;
use crate::ports::UpdateModelSelection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_work: ports::ReadWork,
    pub(super) create_work: create_work::Dependencies,
    pub(super) read_model_allowlist: ReadModelAllowlist,
    pub(super) read_provider_connection: ReadProviderConnection,
    pub(super) read_conversation: ReadConversation,
    pub(super) read_instructions: ports::ReadInstructions,
    pub(super) lock_conversation: LockConversation,
    pub(super) update_conversation: UpdateConversation,
    pub(super) read_models: ReadModels,
    pub(super) create_model_step_factory: CreateModelStepFactory,
    pub(super) update_model_selection: UpdateModelSelection,
    pub(super) read_setup_draft: ports::ReadSetupDraft,
    pub(super) read_file: ports::ReadFile,
    pub(super) create_process: ports::CreateProcess,
    pub(super) update_file: ports::UpdateFile,
    pub(super) create_file: ports::CreateFile,
    pub(super) read_directory: ports::ReadDirectory,
    pub(super) read_paths: ports::ReadPaths,
    pub(super) read_text_matches: ports::ReadTextMatches,
    pub(super) read_code_matches: ports::ReadCodeMatches,
    pub(super) read_web_search: ports::ReadWebSearch,
    pub(super) read_tools: ports::ReadTools,
    pub(super) read_tool_invocation: ports::ReadToolInvocation,
    pub(super) create_tool_output: ports::CreateToolOutput,
}
