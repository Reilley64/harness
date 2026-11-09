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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GuidanceRecord {
    format: u32,
    intent: String,
    parent_id: String,
    parent_agent_id: String,

    work_id: Option<String>,
    work_intent: Option<String>,
    work_version: Option<String>,

    snapshot: SnapshotRecord,
}

#[derive(serde::Serialize)]
struct Reference<'a> {
    format: u32,
    intent: &'a str,
    parent_id: &'a str,
    parent_agent_id: &'a str,
}

pub(super) fn reference(child: &domain::ChildExecution) -> Result<Vec<u8>, Error> {
    encode_bounded(
        &Reference {
            format: 1,
            intent: &child.intent,
            parent_id: &child.parent_id,
            parent_agent_id: &child.parent_agent_id,
        },
        4096,
    )
}

pub(super) fn claim(
    bytes: Option<Vec<u8>>,
    proposed: domain::ChildExecution,
    adoption: Option<Vec<u8>>,
) -> Result<(domain::ChildExecution, Option<Vec<u8>>), Error> {
    proposed.validate()?;
    instructions::validate(&proposed.instructions)?;
    if proposed.stop_requested
        || proposed.status != domain::ChildExecutionStatus::Held
        || proposed.outcome.is_some()
        || proposed.delivered_to.is_some()
        || proposed.follow_up_completed_by.is_some()
    {
        return Err("Initial execution claim must be held without fabricated outcomes".into());
    }
    if let Some(bytes) = bytes {
        let record: IntentRecord = serde_json::from_slice(&bytes)?;
        if record.format != 2 {
            return Err("Unsupported execution claim format".into());
        }
        let existing: domain::ChildExecution = record.execution.try_into()?;
        if !existing.matches_request(&proposed.parent_id, &proposed.brief, proposed.routed) {
            return Err(
                "Dispatch intent is already claimed by a different exact typed payload".into(),
            );
        }
        return Ok((existing, None));
    }
    if let Some(adoption) = adoption {
        let adoption: GuidanceRecord = serde_json::from_slice(&adoption)?;
        let snapshot: domain::InstructionSnapshot = adoption.snapshot.try_into()?;
        if adoption.format != 1
            || adoption.intent != proposed.intent
            || adoption.parent_id != proposed.parent_id
            || adoption.parent_agent_id != proposed.parent_agent_id
            || adoption.work_id != proposed.work_id
            || adoption.work_intent != proposed.work_intent
            || adoption.work_version != proposed.work_version
            || snapshot != proposed.instructions
        {
            return Err("Research adoption changed before claim admission; inspect the exact retained closure".into());
        }
    }
    let bytes = encode_bounded(
        &IntentRecord {
            format: 2,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<Vec<ChildInstructionRecord>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovered: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<ScopeRecord>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ChildInstructionRecord {
    id: String,
    execution_id: String,
    order: u64,
    text: String,

    applied_run: Option<String>,
    applied_reply: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    resume_required: Option<bool>,

    outcome: Option<OutcomeRecord>,

    delivered_to: Option<String>,
    follow_up_completed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<Vec<ReceiptRecord>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiptRecord {
    outcome: OutcomeRecord,
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

impl From<domain::ChildExecution> for ChildRecord {
    fn from(value: domain::ChildExecution) -> Self {
        Self {
            intent: value.intent,
            assignment_id: value.assignment_id,
            agent_id: value.agent_id,
            execution_id: value.execution_id,
            parent_id: value.parent_id,
            parent_agent_id: value.parent_agent_id,
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
            stop_requested: value.stop_requested,
            resume_required: None,
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
            history: (!value.history.is_empty()).then(|| {
                value
                    .history
                    .into_iter()
                    .map(|receipt| ReceiptRecord {
                        outcome: OutcomeRecord {
                            id: receipt.outcome.id,
                            status: status_name(receipt.outcome.status),
                            conversation_id: receipt.outcome.conversation_id,
                            run_id: receipt.outcome.run_id,
                            reply_id: receipt.outcome.reply_id,
                            summary: receipt.outcome.summary,
                            artifacts: receipt.outcome.artifacts,
                        },
                        delivered_to: receipt.delivered_to,
                        follow_up_completed_by: receipt.follow_up_completed_by,
                    })
                    .collect()
            }),
        }
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

impl From<domain::Coordination> for Record {
    fn from(value: domain::Coordination) -> Self {
        let scoped = !value.scopes.is_empty();
        let recovered = scoped || !value.recovered.is_empty();
        let later_runs = recovered || value.children.iter().any(|child| !child.history.is_empty());
        Self {
            format: if scoped {
                6
            } else if recovered {
                5
            } else if later_runs {
                4
            } else if value.instructions.is_empty() {
                2
            } else {
                3
            },
            conversation_id: value.conversation_id,
            agent_id: value.agent_id,
            revision: value.revision,
            authority_complete: value.authority_complete,
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
            waiting: value.waiting,
            recovered: recovered.then_some(value.recovered),
            scopes: scoped.then(|| {
                value
                    .scopes
                    .into_iter()
                    .map(|scope| ScopeRecord {
                        id: scope.id,
                        execution_id: scope.execution_id,
                        version: scope.version,
                        text: scope.text,
                        replaces: scope.replaces,
                        supersedes_run: scope.supersedes_run,
                        approved_by: scope.approved_by,
                    })
                    .collect()
            }),
            instructions: (later_runs || !value.instructions.is_empty()).then(|| {
                value
                    .instructions
                    .into_iter()
                    .map(|instruction| ChildInstructionRecord {
                        id: instruction.id,
                        execution_id: instruction.execution_id,
                        order: instruction.order,
                        text: instruction.text,
                        applied_run: instruction.applied_run,
                        applied_reply: instruction.applied_reply,
                    })
                    .collect()
            }),
            children: value
                .children
                .into_iter()
                .map(|child| {
                    let resume_required = child.resume_required;
                    let mut record = ChildRecord::from(child);
                    if recovered {
                        record.resume_required = Some(resume_required);
                    }
                    if later_runs && record.history.is_none() {
                        record.history = Some(Vec::new());
                    }
                    record
                })
                .collect(),
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
