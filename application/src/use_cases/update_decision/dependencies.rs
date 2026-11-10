use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_decision: ports::ReadDecision,
    pub(super) update_decision: ports::UpdateDecision,
    pub(super) read_decider: ports::ReadDecider,
    pub(super) create_native_decision: ports::CreateNativeDecision,
    pub(super) create_decision_inputs: ports::CreateDecisionInputs,
    pub(super) read_routing_evidence: ports::ReadRoutingEvidence,
    pub(super) read_models: ports::ReadModels,
    pub(super) read_model_allowlist: ports::ReadModelAllowlist,
    pub(super) read_provider_connection: ports::ReadProviderConnection,
    pub(super) create_model_step_factory: ports::CreateModelStepFactory,
    pub(super) read_conversation: ports::ReadConversation,
    pub(super) read_work: ports::ReadWork,
}
