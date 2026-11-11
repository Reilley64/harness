use super::Error;
use crate::conversations;
use sha2::Digest;
use sha2::Sha256;
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    format: u32,
    id: String,
    creator_id: String,
    conversation_id: String,
    workspace: std::path::PathBuf,
    revision: u64,
    publications: Vec<PublicationRecord>,
    approvals: Vec<ApprovalRecord>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicationRecord {
    intent: String,
    proposal: ProposalRecord,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalRecord {
    intent: String,
    version: String,
    revision: u64,
    provider: String,
    model: String,
    effort: Option<String>,
    transient_retry_allowance: Option<u32>,

    coordinator_id: String,
    approved_at: u64,
    delivered: bool,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposalRecord {
    root: String,
    source_revision: String,
    spec: String,
    scope: String,
    criteria: String,
    dependencies: String,
    wiki: Option<String>,
    context: Option<String>,
    artifacts: Vec<String>,
    document: String,
    version: String,
}

impl TryFrom<ProposalRecord> for domain::WorkProposal {
    type Error = Error;
    fn try_from(r: ProposalRecord) -> Result<Self, Error> {
        let p = Self {
            root: r.root,
            source_revision: r.source_revision,
            spec: r.spec,
            scope: r.scope,
            criteria: r.criteria,
            dependencies: r.dependencies,
            wiki: r.wiki,
            context: r.context,
            artifacts: r.artifacts,
            document: r.document,
            version: r.version,
        };
        p.validate()?;
        if format!("{:x}", Sha256::digest(p.document.as_bytes())) != p.version {
            return Err("Proposal document checksum mismatch".into());
        }
        let parsed: serde_json::Value = serde_json::from_str(&p.document)?;
        let actual = serde_json::json!({"root": p.root,"source_revision": p.source_revision,"spec": p.spec,"scope": p.scope,"criteria": p.criteria,"dependencies": p.dependencies,"wiki": p.wiki,"context": p.context,"artifacts": p.artifacts});
        if parsed != actual {
            return Err("Proposal fields differ from the source document".into());
        }
        Ok(p)
    }
}

impl TryFrom<Record> for domain::ApprovedWork {
    type Error = Error;
    fn try_from(r: Record) -> Result<Self, Error> {
        if r.format != 1
            || r.revision == 0
            || !r.workspace.is_absolute()
            || r.publications.is_empty()
            || r.publications.len() > 64
            || r.approvals.len() > 64
        {
            return Err("Invalid approved-work record".into());
        }
        for id in [&r.id, &r.creator_id, &r.conversation_id] {
            conversations::validate_id(id)?;
        }
        let mut seen = std::collections::HashSet::new();
        let mut publications = Vec::new();
        for p in r.publications {
            conversations::validate_id(&p.intent)?;
            if !seen.insert(p.intent.clone()) {
                return Err("Duplicate publication intent".into());
            }
            publications.push(domain::WorkPublication {
                intent: p.intent,
                proposal: p.proposal.try_into()?,
            });
        }
        let mut approvals = Vec::new();
        seen.clear();
        for a in r.approvals {
            conversations::validate_id(&a.intent)?;
            conversations::validate_id(&a.coordinator_id)?;
            if !seen.insert(a.intent.clone())
                || a.revision == 0
                || a.revision >= r.revision
                || !publications.iter().any(|p| p.proposal.version == a.version)
                || a.transient_retry_allowance.is_some_and(|allowance| {
                    allowance > domain::ProviderRetry::MAX_INITIAL_ALLOWANCE
                })
                || a.model.is_empty()
                || a.model.len() > 512
                || !matches!(a.provider.as_str(), "openai" | "openrouter")
            {
                return Err("Invalid approved-work authorization".into());
            }
            let expected =
                uuid::Uuid::new_v5(&uuid::Uuid::parse_str(&r.id)?, a.intent.as_bytes()).to_string();
            if a.coordinator_id != expected {
                return Err("Assignment identity mismatch".into());
            }
            let effort = match a.effort.as_deref() {
                None => None,
                Some("low") => Some(domain::ModelEffort::Low),
                Some("medium") => Some(domain::ModelEffort::Medium),
                Some("high") => Some(domain::ModelEffort::High),
                Some("extra_high") => Some(domain::ModelEffort::ExtraHigh),
                _ => return Err("Invalid approval effort".into()),
            };
            approvals.push(domain::WorkApproval {
                intent: a.intent,
                version: a.version,
                revision: a.revision,
                selection: domain::ModelSelection {
                    provider_id: domain::ProviderId(a.provider),
                    model_id: a.model,
                    effort,
                },
                transient_retry_allowance: a.transient_retry_allowance,
                coordinator_id: a.coordinator_id,
                approved_at: a.approved_at,
                delivered: a.delivered,
            });
        }
        Ok(Self {
            id: r.id,
            creator_id: r.creator_id,
            conversation_id: r.conversation_id,
            workspace: r.workspace,
            revision: r.revision,
            publications,
            approvals,
        })
    }
}
