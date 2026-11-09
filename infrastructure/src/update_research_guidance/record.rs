use crate::conversations;
type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    format: u32,
    intent: String,
    parent_id: String,
    parent_agent_id: String,

    work_id: Option<String>,
    work_intent: Option<String>,
    work_version: Option<String>,

    snapshot: SnapshotRecord,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotRecord {
    id: String,
    phase: String,

    scopes: Vec<String>,
    references: Vec<String>,

    instructions: Vec<InstructionRecord>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct InstructionRecord {
    id: String,
    version: String,
    source: String,
    upstream: Vec<String>,

    requires: Vec<String>,
    body: String,
}
impl TryFrom<Record> for domain::ResearchGuidance {
    type Error = Error;
    fn try_from(value: Record) -> Result<Self, Error> {
        if value.format != 1 {
            return Err("Unsupported Research guidance adoption format".into());
        }
        for id in [&value.intent, &value.parent_id, &value.parent_agent_id] {
            conversations::validate_id(id)?;
        }
        let snapshot = value.snapshot;
        let guidance = Self {
            intent: value.intent,
            parent_id: value.parent_id,
            parent_agent_id: value.parent_agent_id,
            work_id: value.work_id,
            work_intent: value.work_intent,
            work_version: value.work_version,
            snapshot: domain::InstructionSnapshot {
                id: snapshot.id,
                phase: snapshot.phase,
                scopes: snapshot.scopes,
                references: snapshot.references,
                instructions: snapshot
                    .instructions
                    .into_iter()
                    .map(|i| domain::Instruction {
                        id: i.id,
                        version: i.version,
                        source: i.source,
                        upstream: i.upstream,
                        requires: i.requires,
                        body: i.body,
                    })
                    .collect(),
            },
        };
        guidance.validate()?;
        Ok(guidance)
    }
}
impl From<domain::ResearchGuidance> for Record {
    fn from(value: domain::ResearchGuidance) -> Self {
        let snapshot = value.snapshot;
        Self {
            format: 1,
            intent: value.intent,
            parent_id: value.parent_id,
            parent_agent_id: value.parent_agent_id,
            work_id: value.work_id,
            work_intent: value.work_intent,
            work_version: value.work_version,
            snapshot: SnapshotRecord {
                id: snapshot.id,
                phase: snapshot.phase,
                scopes: snapshot.scopes,
                references: snapshot.references,
                instructions: snapshot
                    .instructions
                    .into_iter()
                    .map(|i| InstructionRecord {
                        id: i.id,
                        version: i.version,
                        source: i.source,
                        upstream: i.upstream,
                        requires: i.requires,
                        body: i.body,
                    })
                    .collect(),
            },
        }
    }
}
