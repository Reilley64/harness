use crate::WorkApproval;
use crate::WorkAssignment;
use crate::WorkProposal;
use crate::WorkPublication;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedWork {
    pub id: String,
    pub creator_id: String,
    pub conversation_id: String,
    pub workspace: std::path::PathBuf,
    pub revision: u64,
    pub publications: Vec<WorkPublication>,
    pub approvals: Vec<WorkApproval>,
}
impl ApprovedWork {
    pub fn current(&self) -> Result<&WorkProposal, &'static str> {
        self.publications
            .last()
            .map(|p| &p.proposal)
            .ok_or("Work has no published proposal")
    }
    pub fn publish(
        &mut self,
        intent: String,
        proposal: WorkProposal,
        revision: Option<u64>,
    ) -> Result<bool, &'static str> {
        proposal.validate()?;
        if let Some(prior) = self.publications.iter().find(|p| p.intent == intent) {
            return if prior.proposal == proposal {
                Ok(false)
            } else {
                Err("Publication intent is bound to a different payload")
            };
        }
        if revision != (self.revision != 0).then_some(self.revision)
            || self.publications.len() >= 64
        {
            return Err("Stale work revision or publication history is full");
        }
        if let Ok(current) = self.current()
            && current.root != proposal.root
        {
            return Err("A work identity cannot change its source root");
        }
        self.publications.push(WorkPublication { intent, proposal });
        Ok(true)
    }
    pub fn approve(&mut self, approval: WorkApproval) -> Result<bool, &'static str> {
        if let Some(prior) = self.approvals.iter().find(|a| a.intent == approval.intent) {
            return if prior.version == approval.version
                && prior.revision == approval.revision
                && prior.selection == approval.selection
                && prior.coordinator_id == approval.coordinator_id
            {
                Ok(false)
            } else {
                Err("Assignment intent is bound to a different approval payload")
            };
        }
        if approval.revision != self.revision
            || approval.version != self.current()?.version
            || self.approvals.len() >= 64
        {
            return Err("Approval is stale or assignment history is full");
        }
        self.approvals.push(approval);
        Ok(true)
    }
    pub fn authorize_assignment(
        &self,
        assignment: &WorkAssignment,
        coordinator_id: &str,
    ) -> Result<(), &'static str> {
        let approval = self.authorize(&assignment.intent, &assignment.proposal.version)?;
        if self.id != assignment.work_id
            || self.creator_id != assignment.creator_id
            || approval.coordinator_id != coordinator_id
            || approval.approved_at != assignment.approved_at
            || self.current()? != &assignment.proposal
        {
            return Err("Approved handoff no longer matches backend authority");
        }
        Ok(())
    }
    pub fn authorize(&self, intent: &str, version: &str) -> Result<&WorkApproval, &'static str> {
        if self.current()?.version != version {
            return Err(
                "Approved work changed; inspect and approve the current scope/criteria version",
            );
        }
        self.approvals
            .iter()
            .find(|a| a.intent == intent && a.version == version)
            .ok_or("Matching trusted human authority is missing")
    }
}
