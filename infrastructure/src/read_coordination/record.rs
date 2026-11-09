use super::Error;
use crate::conversations;
use crate::instructions;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    format: u32,
    conversation_id: String,
    agent_id: String,
    revision: u64,
    authority_complete: bool,

    gate: String,

    controller: Option<String>,
    run_id: Option<String>,
    waiting: bool,

    children: Vec<ChildRecord>,
    joins: Vec<JoinRecord>,
    instructions: Option<Vec<ChildInstructionRecord>>,
    recovered: Option<Vec<String>>,
    scopes: Option<Vec<ScopeRecord>>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeRecord {
    id: String,
    execution_id: String,
    version: String,
    text: String,

    replaces: Option<String>,
    supersedes_run: Option<String>,
    approved_by: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ChildInstructionRecord {
    id: String,
    execution_id: String,
    order: u64,
    text: String,

    applied_run: Option<String>,
    applied_reply: Option<String>,
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
    stop_requested: bool,
    resume_required: Option<bool>,

    outcome: Option<OutcomeRecord>,

    delivered_to: Option<String>,
    follow_up_completed_by: Option<String>,
    history: Option<Vec<ReceiptRecord>>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptRecord {
    outcome: OutcomeRecord,
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

#[derive(serde::Deserialize)]
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
            resume_required: value.resume_required.unwrap_or(false),
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
            history: value
                .history
                .unwrap_or_default()
                .into_iter()
                .map(|receipt| {
                    let value = receipt.outcome;
                    Ok::<_, Error>(domain::ChildReceipt {
                        outcome: domain::ChildOutcome {
                            id: value.id,
                            status: status(&value.status)?,
                            conversation_id: value.conversation_id,
                            run_id: value.run_id,
                            reply_id: value.reply_id,
                            summary: value.summary,
                            artifacts: value.artifacts,
                        },
                        delivered_to: receipt.delivered_to,
                        follow_up_completed_by: receipt.follow_up_completed_by,
                    })
                })
                .collect::<Result<_, _>>()?,
        };
        child.validate()?;
        Ok(child)
    }
}

impl TryFrom<Record> for domain::Coordination {
    type Error = Error;
    fn try_from(value: Record) -> Result<Self, Error> {
        if !matches!(
            (value.format, value.instructions.is_some()),
            (2, false) | (3, true) | (4, true) | (5, true) | (6, true)
        ) || value.children.iter().any(|child| {
            child.history.is_some() != (value.format >= 4)
                || child.resume_required.is_some() != (value.format >= 5)
        }) {
            return Err(
                "Unsupported or incomplete coordination format; preserve records and use a compatible executable"
                    .into(),
            );
        }
        if value.recovered.is_some() != (value.format >= 5) {
            return Err("Recovered ownership facts are incomplete or incompatible".into());
        }
        if value.scopes.is_some() != (value.format == 6) {
            return Err("Scope authority facts are incomplete or incompatible".into());
        }
        conversations::validate_id(&value.conversation_id)?;
        conversations::validate_id(&value.agent_id)?;
        let coordination = Self {
            conversation_id: value.conversation_id,
            agent_id: value.agent_id,
            revision: value.revision,
            authority_complete: value.authority_complete,
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
            waiting: value.waiting,
            recovered: value.recovered.unwrap_or_default(),
            scopes: value
                .scopes
                .unwrap_or_default()
                .into_iter()
                .map(|scope| {
                    for id in [&scope.id, &scope.execution_id, &scope.version] {
                        conversations::validate_id(id)?;
                    }
                    for id in [&scope.replaces, &scope.supersedes_run, &scope.approved_by]
                        .into_iter()
                        .flatten()
                    {
                        conversations::validate_id(id)?;
                    }
                    Ok::<_, Error>(domain::ScopeAmendment {
                        id: scope.id,
                        execution_id: scope.execution_id,
                        version: scope.version,
                        text: scope.text,
                        replaces: scope.replaces,
                        supersedes_run: scope.supersedes_run,
                        approved_by: scope.approved_by,
                    })
                })
                .collect::<Result<_, _>>()?,
            instructions: value
                .instructions
                .unwrap_or_default()
                .into_iter()
                .map(|instruction| {
                    conversations::validate_id(&instruction.id)?;
                    conversations::validate_id(&instruction.execution_id)?;
                    if let Some(id) = &instruction.applied_run {
                        conversations::validate_id(id)?;
                    }
                    if let Some(id) = &instruction.applied_reply {
                        conversations::validate_id(id)?;
                    }
                    Ok::<_, Error>(domain::ChildInstruction {
                        id: instruction.id,
                        execution_id: instruction.execution_id,
                        order: instruction.order,
                        text: instruction.text,
                        applied_run: instruction.applied_run,
                        applied_reply: instruction.applied_reply,
                    })
                })
                .collect::<Result<_, _>>()?,
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
