use super::Error;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::instructions;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct IntentRecord {
    format: u32,

    execution: ChildRecord,
}

pub(super) fn claim(
    bytes: Option<Vec<u8>>,
    proposed: domain::ChildExecution,
) -> Result<(domain::ChildExecution, Option<Vec<u8>>), Error> {
    proposed.validate()?;
    instructions::validate(&proposed.instructions)?;
    if proposed.status != domain::ChildExecutionStatus::Held
        || proposed.outcome.is_some()
        || proposed.delivered_to.is_some()
        || proposed.follow_up_completed_by.is_some()
    {
        return Err("Initial execution claim must be held without fabricated outcomes".into());
    }
    if let Some(bytes) = bytes {
        let record: IntentRecord = serde_json::from_slice(&bytes)?;
        if record.format != 1 {
            return Err("Unsupported execution claim format".into());
        }
        let existing: domain::ChildExecution = record.execution.try_into()?;
        if !existing.same_assignment(&proposed) {
            return Err(
                "Dispatch intent is already claimed by a different exact typed payload".into(),
            );
        }
        return Ok((existing, None));
    }
    let bytes = encode_bounded(
        &IntentRecord {
            format: 1,
            execution: proposed.clone().into(),
        },
        4 * 1024 * 1024,
    )?;
    Ok((proposed, Some(bytes)))
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    format: u32,
    agent_id: String,
    revision: u64,

    gate: String,

    controller: Option<String>,
    run_id: Option<String>,

    children: Vec<ChildRecord>,
    joins: Vec<JoinRecord>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ChildRecord {
    intent: String,
    assignment_id: String,
    agent_id: String,
    execution_id: String,

    parent_id: String,
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

    outcome: Option<OutcomeRecord>,

    delivered_to: Option<String>,
    follow_up_completed_by: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PairRecord {
    provider: String,
    model: String,
    effort: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct BriefRecord {
    intent: String,
    role: String,
    task: String,
    constraints: String,
    tool_context: String,
    context_tokens: u64,
}

#[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct CandidateRecord {
    key: String,
    connection_id: String,
    selection: PairRecord,
    model: ModelRecord,
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

#[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct JoinRecord {
    id: String,
    run_id: String,

    mode: String,
    executions: Vec<String>,

    backgrounded: bool,
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

fn effort_name(value: domain::ModelEffort) -> String {
    match value {
        domain::ModelEffort::Low => "low",
        domain::ModelEffort::Medium => "medium",
        domain::ModelEffort::High => "high",
        domain::ModelEffort::ExtraHigh => "extra_high",
    }
    .into()
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

fn status_name(value: domain::ChildExecutionStatus) -> String {
    match value {
        domain::ChildExecutionStatus::Held => "held",
        domain::ChildExecutionStatus::Running => "running",
        domain::ChildExecutionStatus::Completed => "completed",
        domain::ChildExecutionStatus::NeedsHuman => "needs_human",
        domain::ChildExecutionStatus::Failed => "failed",
        domain::ChildExecutionStatus::Stopped => "stopped",
        domain::ChildExecutionStatus::Unknown => "unknown",
    }
    .into()
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

impl From<domain::ModelSelection> for PairRecord {
    fn from(value: domain::ModelSelection) -> Self {
        Self {
            provider: value.provider_id.0,
            model: value.model_id,
            effort: value.effort.map(effort_name),
        }
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

impl From<domain::DecisionCandidate> for CandidateRecord {
    fn from(value: domain::DecisionCandidate) -> Self {
        let model = value.model;
        Self {
            key: value.key,
            connection_id: value.connection_id,
            selection: value.selection.into(),
            model: ModelRecord {
                id: model.id,
                name: model.name,
                provider: model.provider_id.0,
                canonical: model.canonical_id,
                context: model.context_window,
                capabilities: model
                    .capabilities
                    .into_iter()
                    .map(|value| {
                        match value {
                            domain::ModelCapability::Chat => "chat",
                            domain::ModelCapability::Decisions => "decisions",
                        }
                        .into()
                    })
                    .collect(),
                efforts: model.efforts.into_iter().map(effort_name).collect(),
                pricing: model
                    .pricing
                    .map(|prices| [prices.input, prices.cached_input, prices.output]),
            },
        }
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

impl From<domain::InstructionSnapshot> for SnapshotRecord {
    fn from(value: domain::InstructionSnapshot) -> Self {
        Self {
            id: value.id,
            phase: value.phase,
            scopes: value.scopes,
            references: value.references,
            instructions: value
                .instructions
                .into_iter()
                .map(|value| InstructionRecord {
                    id: value.id,
                    version: value.version,
                    source: value.source,
                    upstream: value.upstream,
                    requires: value.requires,
                    body: value.body,
                })
                .collect(),
        }
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

impl From<domain::ChildExecution> for ChildRecord {
    fn from(value: domain::ChildExecution) -> Self {
        Self {
            intent: value.intent,
            assignment_id: value.assignment_id,
            agent_id: value.agent_id,
            execution_id: value.execution_id,
            parent_id: value.parent_id,
            parent_run_id: value.parent_run_id,
            parent_revision: value.parent_revision,
            parent_selection: value.parent_selection.into(),
            parent_connection_id: value.parent_connection_id,
            work_id: value.work_id,
            work_intent: value.work_intent,
            work_version: value.work_version,
            approved_at: value.approved_at,
            brief: BriefRecord {
                intent: value.brief.intent,
                role: value.brief.role,
                task: value.brief.task,
                constraints: value.brief.constraints,
                tool_context: value.brief.tool_context,
                context_tokens: value.brief.context_tokens,
            },
            selected: value.selected.into(),
            instructions: value.instructions.into(),
            routed: value.routed,
            controller: value.controller,
            status: status_name(value.status),
            outcome: value.outcome.map(|value| OutcomeRecord {
                id: value.id,
                status: status_name(value.status),
                conversation_id: value.conversation_id,
                run_id: value.run_id,
                reply_id: value.reply_id,
                summary: value.summary,
                artifacts: value.artifacts,
            }),
            delivered_to: value.delivered_to,
            follow_up_completed_by: value.follow_up_completed_by,
        }
    }
}

impl TryFrom<Record> for domain::Coordination {
    type Error = Error;
    fn try_from(value: Record) -> Result<Self, Error> {
        if value.format != 1 {
            return Err(
                "Unsupported coordination format; preserve records and use a compatible executable"
                    .into(),
            );
        }
        conversations::validate_id(&value.agent_id)?;
        let coordination = Self {
            agent_id: value.agent_id,
            revision: value.revision,
            gate: match value.gate.as_str() {
                "open" => domain::CoordinationGate::Open,
                "needs_human" => domain::CoordinationGate::NeedsHuman,
                "approval" => domain::CoordinationGate::Approval,
                "stopped" => domain::CoordinationGate::Stopped,
                "resume_required" => domain::CoordinationGate::ResumeRequired,
                "unknown" => domain::CoordinationGate::Unknown,
                _ => return Err("Invalid coordination gate".into()),
            },
            controller: value.controller,
            run_id: value.run_id,
            children: value
                .children
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            joins: value
                .joins
                .into_iter()
                .map(|value| {
                    Ok::<_, Error>(domain::CapturedJoin {
                        id: value.id,
                        run_id: value.run_id,
                        mode: match value.mode.as_str() {
                            "any" => domain::JoinMode::Any,
                            "all" => domain::JoinMode::All,
                            _ => return Err("Invalid captured join mode".into()),
                        },
                        executions: value.executions,
                        backgrounded: value.backgrounded,
                    })
                })
                .collect::<Result<_, _>>()?,
        };
        coordination.validate()?;
        Ok(coordination)
    }
}

impl From<domain::Coordination> for Record {
    fn from(value: domain::Coordination) -> Self {
        Self {
            format: 1,
            agent_id: value.agent_id,
            revision: value.revision,
            gate: match value.gate {
                domain::CoordinationGate::Open => "open",
                domain::CoordinationGate::NeedsHuman => "needs_human",
                domain::CoordinationGate::Approval => "approval",
                domain::CoordinationGate::Stopped => "stopped",
                domain::CoordinationGate::ResumeRequired => "resume_required",
                domain::CoordinationGate::Unknown => "unknown",
            }
            .into(),
            controller: value.controller,
            run_id: value.run_id,
            children: value.children.into_iter().map(Into::into).collect(),
            joins: value
                .joins
                .into_iter()
                .map(|value| JoinRecord {
                    id: value.id,
                    run_id: value.run_id,
                    mode: match value.mode {
                        domain::JoinMode::Any => "any",
                        domain::JoinMode::All => "all",
                    }
                    .into(),
                    executions: value.executions,
                    backgrounded: value.backgrounded,
                })
                .collect(),
        }
    }
}
