use crate::ports;

use crate::ports::CreateModelStepFactory;
use crate::ports::LockConversation;
use crate::ports::ReadConversation;
use crate::ports::ReadModels;
use crate::ports::UpdateConversation;
use crate::ports::UpdateModelSelection;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_conversation: ReadConversation,
    pub(super) lock_conversation: LockConversation,
    pub(super) update_conversation: UpdateConversation,
    pub(super) read_models: ReadModels,
    pub(super) create_model_step_factory: CreateModelStepFactory,
    pub(super) update_model_selection: UpdateModelSelection,
    pub(super) read_file: ports::ReadFile,
    pub(super) create_process: ports::CreateProcess,
    pub(super) update_file: ports::UpdateFile,
    pub(super) create_file: ports::CreateFile,
    pub(super) read_directory: ports::ReadDirectory,
    pub(super) read_paths: ports::ReadPaths,
    pub(super) read_text_matches: ports::ReadTextMatches,
    pub(super) read_code_matches: ports::ReadCodeMatches,
    pub(super) read_web_search: ports::ReadWebSearch,
    pub(super) update_mcp_configuration: ports::UpdateMcpConfiguration,
    pub(super) create_mcp_call: ports::CreateMcpCall,
    pub(super) read_tools: ports::ReadTools,
    pub(super) read_tool_invocation: ports::ReadToolInvocation,
    pub(super) create_tool_output: ports::CreateToolOutput,
}
