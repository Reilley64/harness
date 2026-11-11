use crate::ports;

#[fundle::deps]
#[derive(Clone)]
pub struct Dependencies {
    pub(super) read_decision: ports::ReadDecision,
    pub(super) read_routing_evidence: ports::ReadRoutingEvidence,
}
