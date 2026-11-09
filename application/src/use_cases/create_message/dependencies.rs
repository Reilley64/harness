use crate::child_runtime::ChildRuntime;
use crate::ports;
use crate::ports::ReadModelAllowlist;
use crate::ports::ReadProviderConnection;
use crate::use_cases::create_assignment;
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
    pub(crate) child_runtime: ChildRuntime,
    pub(crate) create_execution_diagnostics: ports::CreateExecutionDiagnostics,
    pub(crate) create_research: ports::CreateResearch,
    pub(crate) read_coordination: ports::ReadCoordination,
    pub(crate) read_execution_claim: ports::ReadExecutionClaim,
    pub(crate) read_execution_claims: ports::ReadExecutionClaims,
    pub(crate) read_research_guidance: ports::ReadResearchGuidance,
    pub(crate) update_research_guidance: ports::UpdateResearchGuidance,
    pub(crate) update_coordination: ports::UpdateCoordination,
    pub(crate) read_decision: ports::ReadDecision,

    pub(crate) read_work: ports::ReadWork,
    pub(super) create_work: create_work::Dependencies,
    pub(super) create_assignment: create_assignment::Dependencies,

    pub(crate) read_model_allowlist: ReadModelAllowlist,
    pub(crate) read_provider_connection: ReadProviderConnection,
    pub(crate) read_conversation: ReadConversation,
    pub(crate) read_instructions: ports::ReadInstructions,
    pub(super) lock_conversation: LockConversation,
    pub(super) update_conversation: UpdateConversation,
    pub(crate) read_models: ReadModels,
    pub(crate) create_model_step_factory: CreateModelStepFactory,
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

    pub(super) update_mcp_configuration: ports::UpdateMcpConfiguration,
    pub(super) create_mcp_call: ports::CreateMcpCall,

    pub(super) read_tools: ports::ReadTools,
    pub(super) read_tool_invocation: ports::ReadToolInvocation,
    pub(super) create_tool_output: ports::CreateToolOutput,
}
