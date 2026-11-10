macro_rules! port {
    ($visibility:vis $name:ident($($argument:ident: $argument_type:ty),* $(,)?) -> $output:ty;) => {
        #[derive(Clone)]
        $visibility struct $name(
            std::sync::Arc<dyn Fn($($argument_type),*) -> $output + Send + Sync>,
        );

        impl $name {
            pub fn new(
                implementation: impl Fn($($argument_type),*) -> $output + Send + Sync + 'static,
            ) -> Self {
                Self(std::sync::Arc::new(implementation))
            }

            pub fn call(&self $(, $argument: $argument_type)*) -> $output {
                (self.0)($($argument),*)
            }
        }
    };
}

mod lock_execution;
pub use lock_execution::ExecutionGuard;
pub use lock_execution::LockExecution;
pub use lock_execution::LockExecutionError;

mod create_execution_diagnostics;
pub use create_execution_diagnostics::CreateExecutionDiagnostics;
pub use create_execution_diagnostics::CreateExecutionDiagnosticsError;
mod create_research;
pub use create_research::CreateResearch;
pub use create_research::CreateResearchError;
mod read_coordination;
mod read_execution_claim;
mod read_execution_claims;
pub use read_execution_claims::ReadExecutionClaims;
pub use read_execution_claims::ReadExecutionClaimsError;
mod update_research_guidance;
pub use update_research_guidance::UpdateResearchGuidance;
pub use update_research_guidance::UpdateResearchGuidanceError;
mod read_research_guidance;
pub use read_research_guidance::ReadResearchGuidance;
pub use read_research_guidance::ReadResearchGuidanceError;
mod update_coordination;
pub use read_coordination::ReadCoordination;
pub use read_coordination::ReadCoordinationError;
pub use read_execution_claim::ReadExecutionClaim;
pub use read_execution_claim::ReadExecutionClaimError;
pub use update_coordination::Response as CoordinationResponse;
pub use update_coordination::UpdateCoordination;
pub use update_coordination::UpdateCoordinationError;
mod read_decision;
pub use read_decision::ReadDecision;
pub use read_decision::ReadDecisionError;
mod update_decision;
pub use update_decision::UpdateDecision;
pub use update_decision::UpdateDecisionError;
mod create_native_decision;
pub use create_native_decision::CreateNativeDecision;
pub use create_native_decision::CreateNativeDecisionError;
mod read_decider;
pub use read_decider::ReadDecider;
pub use read_decider::ReadDeciderError;
mod update_decider;
pub use update_decider::UpdateDecider;
pub use update_decider::UpdateDeciderError;

mod read_work;
pub use read_work::ReadWork;
pub use read_work::ReadWorkError;
mod update_work;
pub use update_work::UpdateWork;
pub use update_work::UpdateWorkError;
mod read_work_proposal;
pub use read_work_proposal::ReadWorkProposal;
pub use read_work_proposal::ReadWorkProposalError;
mod create_coordinator;
pub use create_coordinator::CreateCoordinator;
pub use create_coordinator::CreateCoordinatorError;
mod read_instructions;
pub use read_instructions::ReadInstructions;
pub use read_instructions::ReadInstructionsError;

mod read_setup_draft;
pub use read_setup_draft::*;
mod update_setup_draft;
pub use update_setup_draft::*;
mod read_setup;
pub use read_setup::*;
mod read_agent;
pub use read_agent::ReadAgent;
pub use read_agent::ReadAgentError;
mod create_agent;
pub use create_agent::CreateAgent;
pub use create_agent::CreateAgentError;
mod create_conversation;
mod create_model_step;
mod create_model_step_factory;
mod lock_conversation;
mod present_turn_event;
mod read_conversation;
mod read_provider_connection;
mod update_conversation;

mod create_provider_connection;
mod create_provider_connection_factory;
mod present_authorization;
mod read_model_allowlist;
mod update_model_allowlist;
pub use read_model_allowlist::ReadModelAllowlist;
pub use read_model_allowlist::ReadModelAllowlistError;
pub use update_model_allowlist::ModelAllowlistUpdate;
pub use update_model_allowlist::UpdateModelAllowlist;
pub use update_model_allowlist::UpdateModelAllowlistError;
mod read_model_selection;
mod read_models;
mod read_providers;
mod update_model_selection;

pub use create_conversation::CreateConversation;
pub use create_conversation::CreateConversationError;
pub use create_model_step::CreateModelStep;
pub use create_model_step::CreateModelStepError;
pub use create_model_step_factory::CreateModelStepFactory;
pub use create_model_step_factory::CreateModelStepFactoryError;
pub use lock_conversation::ConversationGuard;
pub use lock_conversation::LockConversation;
pub use lock_conversation::LockConversationError;
pub use present_turn_event::PresentTurnEvent;
pub use read_conversation::ReadConversation;
pub use read_conversation::ReadConversationError;
pub use read_provider_connection::ReadProviderConnection;
pub use read_provider_connection::ReadProviderConnectionError;
pub use update_conversation::UpdateConversation;
pub use update_conversation::UpdateConversationError;

pub use create_provider_connection::CreateProviderConnection;
pub use create_provider_connection::CreateProviderConnectionError;
pub use create_provider_connection_factory::CreateProviderConnectionFactory;
pub use create_provider_connection_factory::CreateProviderConnectionFactoryError;
pub use present_authorization::AuthorizationChallenge;
pub use present_authorization::PresentAuthorization;
pub use read_model_selection::ReadModelSelection;
pub use read_model_selection::ReadModelSelectionError;
pub use read_models::ReadModels;
pub use read_models::ReadModelsError;
pub use read_providers::ReadProviders;
pub use read_providers::ReadProvidersError;
pub use update_model_selection::UpdateModelSelection;
pub use update_model_selection::UpdateModelSelectionError;

mod read_file;
pub use read_file::ReadFile;
pub use read_file::ReadFileError;
mod create_process;
pub use create_process::CreateProcess;
pub use create_process::CreateProcessError;
mod update_file;
pub use update_file::UpdateFile;
pub use update_file::UpdateFileError;
mod create_file;
pub use create_file::CreateFile;
pub use create_file::CreateFileError;
mod read_directory;
pub use read_directory::ReadDirectory;
pub use read_directory::ReadDirectoryError;
mod read_paths;
pub use read_paths::ReadPaths;
pub use read_paths::ReadPathsError;
mod read_text_matches;
pub use read_text_matches::ReadTextMatches;
pub use read_text_matches::ReadTextMatchesError;
mod read_code_matches;
pub use read_code_matches::ReadCodeMatches;
pub use read_code_matches::ReadCodeMatchesError;
mod read_web_search;
pub use read_web_search::ReadWebSearch;
pub use read_web_search::ReadWebSearchError;

mod read_tool_invocation;
pub use read_tool_invocation::ReadToolInvocation;
pub use read_tool_invocation::ReadToolInvocationError;
mod read_tools;
pub use read_tools::ReadTools;
pub use read_tools::ReadToolsError;
mod create_tool_output;
pub use create_tool_output::CreateToolOutput;
pub use create_tool_output::CreateToolOutputError;
use futures::future::BoxFuture;

pub type PortFuture<T> = BoxFuture<'static, T>;
mod present_mcp_authorization;
pub use present_mcp_authorization::PresentMcpAuthorization;
mod update_mcp_configuration;
pub use update_mcp_configuration::Response as McpConfigurationResponse;
pub use update_mcp_configuration::UpdateMcpConfiguration;
pub use update_mcp_configuration::UpdateMcpConfigurationError;
mod create_mcp_call;
pub use create_mcp_call::CreateMcpCall;
pub use create_mcp_call::CreateMcpCallError;
pub use create_mcp_call::Response as McpCallResponse;
