use super::Error;
use crate::conversations;
use crate::instructions;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    format: u32,
    execution: ChildRecord,
}

impl TryFrom<Record> for domain::ChildExecution {
    type Error = Error;

    fn try_from(value: Record) -> Result<Self, Error> {
        if value.format != 2 {
            return Err(
                "Unsupported execution claim format; preserve it and use a compatible executable"
                    .into(),
            );
        }
        value.execution.try_into()
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ChildRecord {
    intent: String,
    assignment_id: String,
    agent_id: String,
    execution_id: String,

    parent_id: String,
    parent_agent_id: String,
    parent_run_id: String,
    parent_revision: u64,

    parent_selection: PairRecord,
    parent_connection_id: String,

    work_id: Option<String>,
    work_intent: Option<String>,
    work_version: Option<String>,
    approved_at: Option<u64>,

    brief: BriefRecord,
    selected: CandidateRecord,
    instructions: SnapshotRecord,
    routed: bool,

    controller: String,
    status: String,
    #[serde(default)]
    stop_requested: bool,

    outcome: Option<OutcomeRecord>,

    delivered_to: Option<String>,
    follow_up_completed_by: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PairRecord {
    provider: String,
    model: String,
    effort: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BriefRecord {
    intent: String,
    role: String,
    task: String,
    constraints: String,
    tool_context: String,
    context_tokens: u64,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRecord {
    id: String,
    name: String,
    provider: String,
    canonical: Option<String>,
    context: Option<u64>,
    capabilities: Vec<String>,
    efforts: Vec<String>,
    pricing: Option<[Option<f64>; 3]>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateRecord {
    key: String,
    connection_id: String,
    selection: PairRecord,
    model: ModelRecord,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotRecord {
    id: String,
    phase: String,

    scopes: Vec<String>,
    references: Vec<String>,

    instructions: Vec<InstructionRecord>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InstructionRecord {
    id: String,
    version: String,
    source: String,
    upstream: Vec<String>,

    requires: Vec<String>,
    body: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomeRecord {
    id: String,
    status: String,
    conversation_id: String,
    run_id: Option<String>,
    reply_id: Option<String>,

    summary: String,
    artifacts: Vec<String>,
}

fn effort(value: &str) -> Result<domain::ModelEffort, Error> {
    Ok(match value {
        "low" => domain::ModelEffort::Low,
        "medium" => domain::ModelEffort::Medium,
        "high" => domain::ModelEffort::High,
        "extra_high" => domain::ModelEffort::ExtraHigh,
        _ => return Err("Invalid coordination thinking selection".into()),
    })
}

fn status(value: &str) -> Result<domain::ChildExecutionStatus, Error> {
    Ok(match value {
        "held" => domain::ChildExecutionStatus::Held,
        "running" => domain::ChildExecutionStatus::Running,
        "completed" => domain::ChildExecutionStatus::Completed,
        "needs_human" => domain::ChildExecutionStatus::NeedsHuman,
        "failed" => domain::ChildExecutionStatus::Failed,
        "stopped" => domain::ChildExecutionStatus::Stopped,
        "unknown" => domain::ChildExecutionStatus::Unknown,
        _ => return Err("Invalid child execution status".into()),
    })
}

impl TryFrom<PairRecord> for domain::ModelSelection {
    type Error = Error;
    fn try_from(value: PairRecord) -> Result<Self, Error> {
        Ok(Self {
            provider_id: domain::ProviderId(value.provider),
            model_id: value.model,
            effort: value.effort.as_deref().map(effort).transpose()?,
        })
    }
}

impl TryFrom<CandidateRecord> for domain::DecisionCandidate {
    type Error = Error;
    fn try_from(value: CandidateRecord) -> Result<Self, Error> {
        let model = value.model;
        if model.pricing.as_ref().is_some_and(|prices| {
            prices
                .iter()
                .flatten()
                .any(|price| !price.is_finite() || *price < 0.0)
        }) {
            return Err("Invalid coordination price evidence".into());
        }
        Ok(Self {
            key: value.key,
            connection_id: value.connection_id,
            selection: value.selection.try_into()?,
            model: domain::Model {
                id: model.id,
                name: model.name,
                provider_id: domain::ProviderId(model.provider),
                canonical_id: model.canonical,
                context_window: model.context,
                capabilities: model
                    .capabilities
                    .into_iter()
                    .map(|value| match value.as_str() {
                        "chat" => Ok(domain::ModelCapability::Chat),
                        "decisions" => Ok(domain::ModelCapability::Decisions),
                        _ => Err::<_, Error>("Invalid coordination capability".into()),
                    })
                    .collect::<Result<_, _>>()?,
                efforts: model
                    .efforts
                    .iter()
                    .map(|value| effort(value))
                    .collect::<Result<_, _>>()?,
                pricing: model.pricing.map(|prices| domain::ModelPricing {
                    input: prices[0],
                    cached_input: prices[1],
                    output: prices[2],
                }),
            },
        })
    }
}

impl TryFrom<SnapshotRecord> for domain::InstructionSnapshot {
    type Error = Error;
    fn try_from(value: SnapshotRecord) -> Result<Self, Error> {
        let snapshot = Self {
            id: value.id,
            phase: value.phase,
            scopes: value.scopes,
            references: value.references,
            instructions: value
                .instructions
                .into_iter()
                .map(|value| domain::Instruction {
                    id: value.id,
                    version: value.version,
                    source: value.source,
                    upstream: value.upstream,
                    requires: value.requires,
                    body: value.body,
                })
                .collect(),
        };
        instructions::validate(&snapshot)?;
        Ok(snapshot)
    }
}

impl TryFrom<ChildRecord> for domain::ChildExecution {
    type Error = Error;
    fn try_from(value: ChildRecord) -> Result<Self, Error> {
        for id in [
            &value.intent,
            &value.assignment_id,
            &value.agent_id,
            &value.execution_id,
            &value.parent_id,
            &value.parent_agent_id,
            &value.parent_run_id,
            &value.controller,
        ] {
            conversations::validate_id(id)?;
        }
        let child = Self {
            intent: value.intent,
            assignment_id: value.assignment_id,
            agent_id: value.agent_id,
            execution_id: value.execution_id,
            parent_id: value.parent_id,
            parent_agent_id: value.parent_agent_id,
            parent_run_id: value.parent_run_id,
            parent_revision: value.parent_revision,
            parent_selection: value.parent_selection.try_into()?,
            parent_connection_id: value.parent_connection_id,
            work_id: value.work_id,
            work_intent: value.work_intent,
            work_version: value.work_version,
            approved_at: value.approved_at,
            brief: domain::AssignmentBrief {
                intent: value.brief.intent,
                role: value.brief.role,
                task: value.brief.task,
                constraints: value.brief.constraints,
                tool_context: value.brief.tool_context,
                context_tokens: value.brief.context_tokens,
            },
            selected: value.selected.try_into()?,
            instructions: value.instructions.try_into()?,
            routed: value.routed,
            controller: value.controller,
            status: status(&value.status)?,
            stop_requested: value.stop_requested,
            outcome: value
                .outcome
                .map(|value| {
                    Ok::<_, Error>(domain::ChildOutcome {
                        id: value.id,
                        status: status(&value.status)?,
                        conversation_id: value.conversation_id,
                        run_id: value.run_id,
                        reply_id: value.reply_id,
                        summary: value.summary,
                        artifacts: value.artifacts,
                    })
                })
                .transpose()?,
            delivered_to: value.delivered_to,
            follow_up_completed_by: value.follow_up_completed_by,
        };
        child.validate()?;
        Ok(child)
    }
}
