use super::Error;
use crate::conversations;
use crate::create_decision_inputs;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    format: u32,
    id: String,
    parent_id: String,
    parent_agent_id: String,
    parent_revision: u64,
    parent: PairRecord,
    parent_connection_id: String,
    work_id: String,
    work_intent: String,
    work_version: String,
    brief: BriefRecord,
    revision: u64,
    finished_at: Option<u64>,
    phase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pool_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stage_state: Option<String>,
    request: Option<RequestRecord>,
    response: Option<ResponseRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stages: Option<Vec<StageRecord>>,
    selected: Option<CandidateRecord>,
    reason: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BriefRecord {
    intent: String,
    role: String,
    task: String,
    constraints: String,
    tool_context: String,
    context_tokens: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PairRecord {
    provider: String,
    model: String,
    effort: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRecord {
    id: String,
    provider: String,
    canonical: Option<String>,
    name: String,
    context: Option<u64>,
    capabilities: Vec<String>,
    efforts: Vec<String>,
    pricing: Option<[Option<f64>; 3]>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateRecord {
    key: String,
    selection: PairRecord,
    connection_id: String,
    model: ModelRecord,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestRecord {
    model: ModelRecord,
    connection_id: String,
    candidates: Vec<CandidateRecord>,
    captured_at: u64,
    policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence: Option<Vec<(String, Vec<EvidenceRecord>)>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseRecord {
    choice: Option<String>,
    confidence: Option<f64>,
    probabilities: Vec<(String, f64)>,
    resolved_model: String,
    serving_provider: Option<String>,
    input_tokens: u64,
    output_tokens: u64,
    cost: Option<f64>,
}

fn effort(e: domain::ModelEffort) -> String {
    match e {
        domain::ModelEffort::Low => "low",
        domain::ModelEffort::Medium => "medium",
        domain::ModelEffort::High => "high",
        domain::ModelEffort::ExtraHigh => "extra_high",
    }
    .into()
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StageRecord {
    status: String,
    started_at: Option<u64>,
    id: String,
    level: u32,
    members: Vec<String>,
    input: String,
    response: Option<ResponseRecord>,
    failure: Option<String>,
    finished_at: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceRecord {
    category: String,
    metric: String,
    value: Option<f64>,
    unit: String,
    source: String,
    source_url: String,
    revision: Option<String>,
    retrieved_at: Option<u64>,
    measured_at: Option<u64>,
    published_at: Option<u64>,
    effective_at: Option<u64>,
    refresh_failed: bool,
    requested_model: String,
    resolved_model: Option<String>,
    serving_provider: Option<String>,
    effort: Option<String>,
    configuration: String,
}

impl TryFrom<EvidenceRecord> for domain::RoutingEvidence {
    type Error = Error;
    fn try_from(e: EvidenceRecord) -> Result<Self, Error> {
        let value = Self {
            category: e.category,
            metric: e.metric,
            value: e.value,
            unit: e.unit,
            source: e.source,
            source_url: e.source_url,
            revision: e.revision,
            retrieved_at: e.retrieved_at,
            measured_at: e.measured_at,
            published_at: e.published_at,
            effective_at: e.effective_at,
            refresh_failed: e.refresh_failed,
            requested_model: e.requested_model,
            resolved_model: e.resolved_model,
            serving_provider: e.serving_provider,
            effort: e.effort.as_deref().map(parse_effort).transpose()?,
            configuration: e.configuration,
        };
        value.validate()?;
        Ok(value)
    }
}

impl From<ResponseRecord> for domain::DecisionResponse {
    fn from(r: ResponseRecord) -> Self {
        Self {
            choice: r.choice,
            confidence: r.confidence,
            probabilities: r.probabilities,
            resolved_model: r.resolved_model,
            serving_provider: r.serving_provider,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost: r.cost,
        }
    }
}

impl TryFrom<StageRecord> for domain::DecisionStage {
    type Error = Error;
    fn try_from(s: StageRecord) -> Result<Self, Error> {
        let stage = Self {
            id: s.id,
            level: s.level,
            members: s.members,
            input: s.input,
            started_at: s.started_at,
            response: s.response.map(Into::into),
            failure: s.failure,
            finished_at: s.finished_at,
        };
        if s.status != stage.status() {
            return Err("Native stage lifecycle status differs from its explicit facts".into());
        }
        Ok(stage)
    }
}

impl From<domain::RoutingEvidence> for EvidenceRecord {
    fn from(e: domain::RoutingEvidence) -> Self {
        Self {
            category: e.category,
            metric: e.metric,
            value: e.value,
            unit: e.unit,
            source: e.source,
            source_url: e.source_url,
            revision: e.revision,
            retrieved_at: e.retrieved_at,
            measured_at: e.measured_at,
            published_at: e.published_at,
            effective_at: e.effective_at,
            refresh_failed: e.refresh_failed,
            requested_model: e.requested_model,
            resolved_model: e.resolved_model,
            serving_provider: e.serving_provider,
            effort: e.effort.map(effort),
            configuration: e.configuration,
        }
    }
}

impl From<domain::DecisionResponse> for ResponseRecord {
    fn from(r: domain::DecisionResponse) -> Self {
        Self {
            choice: r.choice,
            confidence: r.confidence,
            probabilities: r.probabilities,
            resolved_model: r.resolved_model,
            serving_provider: r.serving_provider,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost: r.cost,
        }
    }
}

impl From<domain::DecisionStage> for StageRecord {
    fn from(s: domain::DecisionStage) -> Self {
        Self {
            status: s.status().into(),
            started_at: s.started_at,
            id: s.id,
            level: s.level,
            members: s.members,
            input: s.input,
            response: s.response.map(Into::into),
            failure: s.failure,
            finished_at: s.finished_at,
        }
    }
}

fn parse_effort(e: &str) -> Result<domain::ModelEffort, Error> {
    Ok(match e {
        "low" => domain::ModelEffort::Low,
        "medium" => domain::ModelEffort::Medium,
        "high" => domain::ModelEffort::High,
        "extra_high" => domain::ModelEffort::ExtraHigh,
        _ => return Err("Invalid stored effort".into()),
    })
}

impl From<domain::ModelSelection> for PairRecord {
    fn from(p: domain::ModelSelection) -> Self {
        Self {
            provider: p.provider_id.0,
            model: p.model_id,
            effort: p.effort.map(effort),
        }
    }
}

impl TryFrom<PairRecord> for domain::ModelSelection {
    type Error = Error;
    fn try_from(p: PairRecord) -> Result<Self, Error> {
        if p.model.is_empty()
            || p.model.len() > 512
            || !matches!(p.provider.as_str(), "openai" | "openrouter")
        {
            return Err("Invalid stored model pair".into());
        }
        Ok(Self {
            provider_id: domain::ProviderId(p.provider),
            model_id: p.model,
            effort: p.effort.as_deref().map(parse_effort).transpose()?,
        })
    }
}

impl From<domain::Model> for ModelRecord {
    fn from(m: domain::Model) -> Self {
        Self {
            id: m.id,
            provider: m.provider_id.0,
            canonical: m.canonical_id,
            name: m.name,
            context: m.context_window,
            capabilities: m
                .capabilities
                .into_iter()
                .map(|c| {
                    match c {
                        domain::ModelCapability::Chat => "chat",
                        domain::ModelCapability::Decisions => "decisions",
                    }
                    .into()
                })
                .collect(),
            efforts: m.efforts.into_iter().map(effort).collect(),
            pricing: m.pricing.map(|p| [p.input, p.cached_input, p.output]),
        }
    }
}

impl TryFrom<ModelRecord> for domain::Model {
    type Error = Error;
    fn try_from(m: ModelRecord) -> Result<Self, Error> {
        if m.pricing
            .as_ref()
            .is_some_and(|p| p.iter().flatten().any(|n| !n.is_finite() || *n < 0.0))
        {
            return Err("Invalid stored pricing".into());
        }
        Ok(Self {
            id: m.id,
            provider_id: domain::ProviderId(m.provider),
            canonical_id: m.canonical,
            name: m.name,
            context_window: m.context,
            capabilities: m
                .capabilities
                .into_iter()
                .map(|c| {
                    Ok(match c.as_str() {
                        "chat" => domain::ModelCapability::Chat,
                        "decisions" => domain::ModelCapability::Decisions,
                        _ => return Err::<_, Error>("Invalid capability".into()),
                    })
                })
                .collect::<Result<_, Error>>()?,
            efforts: m
                .efforts
                .iter()
                .map(|e| parse_effort(e))
                .collect::<Result<_, _>>()?,
            pricing: m.pricing.map(|p| domain::ModelPricing {
                input: p[0],
                cached_input: p[1],
                output: p[2],
            }),
        })
    }
}

impl From<domain::DecisionCandidate> for CandidateRecord {
    fn from(c: domain::DecisionCandidate) -> Self {
        Self {
            key: c.key,
            selection: c.selection.into(),
            connection_id: c.connection_id,
            model: c.model.into(),
        }
    }
}

impl TryFrom<CandidateRecord> for domain::DecisionCandidate {
    type Error = Error;
    fn try_from(c: CandidateRecord) -> Result<Self, Error> {
        Ok(Self {
            key: c.key,
            selection: c.selection.try_into()?,
            connection_id: c.connection_id,
            model: c.model.try_into()?,
        })
    }
}

impl From<domain::AssignmentDecision> for Record {
    fn from(d: domain::AssignmentDecision) -> Self {
        Self {
            format: if d.legacy { 1 } else { 2 },
            pool_state: (!d.legacy).then(|| {
                if d.request.is_some() {
                    "complete_pool"
                } else {
                    "not_captured"
                }
                .into()
            }),
            stage_state: (!d.legacy).then(|| {
                if d.request.is_none() {
                    "not_requested"
                } else if d.stages.is_empty() {
                    "unsupported_before_admission"
                } else {
                    "planned"
                }
                .into()
            }),
            id: d.id,
            parent_id: d.parent_id,
            parent_agent_id: d.parent_agent_id,
            parent_revision: d.parent_revision,
            parent: d.parent.into(),
            parent_connection_id: d.parent_connection_id,
            work_id: d.work_id,
            work_intent: d.work_intent,
            work_version: d.work_version,
            brief: BriefRecord {
                intent: d.brief.intent,
                role: d.brief.role,
                task: d.brief.task,
                constraints: d.brief.constraints,
                tool_context: d.brief.tool_context,
                context_tokens: d.brief.context_tokens,
            },
            revision: d.revision,
            finished_at: d.finished_at,
            phase: match d.phase {
                domain::DecisionPhase::Prepared => "prepared",
                domain::DecisionPhase::Pending => "pending",
                domain::DecisionPhase::Selected => "selected",
                domain::DecisionPhase::Fallback => "fallback",
                domain::DecisionPhase::Configuration => "configuration",
                domain::DecisionPhase::Unsupported => "unsupported",
                domain::DecisionPhase::Stopped => "stopped",
            }
            .into(),
            request: d.request.map(|r| RequestRecord {
                model: r.model.into(),
                connection_id: r.connection_id,
                candidates: r.candidates.into_iter().map(Into::into).collect(),
                captured_at: r.captured_at,
                policy: r.policy,
                evidence: (!d.legacy).then(|| {
                    r.evidence
                        .into_iter()
                        .map(|(key, facts)| (key, facts.into_iter().map(Into::into).collect()))
                        .collect()
                }),
            }),
            response: d.response.map(|r| ResponseRecord {
                choice: r.choice,
                confidence: r.confidence,
                probabilities: r.probabilities,
                resolved_model: r.resolved_model,
                serving_provider: r.serving_provider,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cost: r.cost,
            }),
            stages: (!d.legacy).then(|| d.stages.into_iter().map(Into::into).collect()),
            selected: d.selected.map(Into::into),
            reason: d.reason,
        }
    }
}

impl TryFrom<Record> for domain::AssignmentDecision {
    type Error = Error;
    fn try_from(r: Record) -> Result<Self, Error> {
        if !matches!(r.format, 1 | 2)
            || r.revision == 0
            || r.format == 1
                && (r.stages.is_some() || r.pool_state.is_some() || r.stage_state.is_some())
            || r.format == 2 && (r.pool_state.is_none() || r.stage_state.is_none())
            || r.format == 2 && r.stages.is_none()
            || r.request
                .as_ref()
                .is_some_and(|q| (r.format == 1) == q.evidence.is_some())
        {
            return Err("Unsupported decision record".into());
        }
        for id in [
            &r.id,
            &r.parent_id,
            &r.parent_agent_id,
            &r.work_id,
            &r.work_intent,
        ] {
            conversations::validate_id(id)?;
        }
        let legacy = r.format == 1;
        let d = Self {
            legacy,
            id: r.id,
            parent_id: r.parent_id,
            parent_agent_id: r.parent_agent_id,
            parent_revision: r.parent_revision,
            parent: r.parent.try_into()?,
            parent_connection_id: r.parent_connection_id,
            work_id: r.work_id,
            work_intent: r.work_intent,
            work_version: r.work_version,
            brief: domain::AssignmentBrief {
                intent: r.brief.intent,
                role: r.brief.role,
                task: r.brief.task,
                constraints: r.brief.constraints,
                tool_context: r.brief.tool_context,
                context_tokens: r.brief.context_tokens,
            },
            revision: r.revision,
            finished_at: r.finished_at,
            phase: match r.phase.as_str() {
                "prepared" => domain::DecisionPhase::Prepared,
                "pending" => domain::DecisionPhase::Pending,
                "selected" => domain::DecisionPhase::Selected,
                "fallback" => domain::DecisionPhase::Fallback,
                "configuration" => domain::DecisionPhase::Configuration,
                "unsupported" => domain::DecisionPhase::Unsupported,
                "stopped" => domain::DecisionPhase::Stopped,
                _ => return Err("Invalid decision phase".into()),
            },
            request: r
                .request
                .map(|r| {
                    Ok::<_, Error>(domain::DecisionRequest {
                        model: r.model.try_into()?,
                        connection_id: r.connection_id,
                        candidates: r
                            .candidates
                            .into_iter()
                            .map(TryInto::try_into)
                            .collect::<Result<_, _>>()?,
                        captured_at: r.captured_at,
                        policy: r.policy,
                        evidence: r
                            .evidence
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(key, facts)| {
                                Ok::<_, Error>((
                                    key,
                                    facts
                                        .into_iter()
                                        .map(TryInto::try_into)
                                        .collect::<Result<_, _>>()?,
                                ))
                            })
                            .collect::<Result<_, _>>()?,
                    })
                })
                .transpose()?,
            response: r.response.map(|r| domain::DecisionResponse {
                choice: r.choice,
                confidence: r.confidence,
                probabilities: r.probabilities,
                resolved_model: r.resolved_model,
                serving_provider: r.serving_provider,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cost: r.cost,
            }),
            stages: r
                .stages
                .unwrap_or_default()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            selected: r.selected.map(TryInto::try_into).transpose()?,
            reason: r.reason,
        };
        d.validate()?;
        if !legacy {
            let pool_state = if d.request.is_some() {
                "complete_pool"
            } else {
                "not_captured"
            };
            let stage_state = if d.request.is_none() {
                "not_requested"
            } else if d.stages.is_empty() {
                "unsupported_before_admission"
            } else {
                "planned"
            };
            if r.pool_state.as_deref() != Some(pool_state)
                || r.stage_state.as_deref() != Some(stage_state)
            {
                return Err(
                    "Explicit complete-pool lifecycle authority differs from the stored facts"
                        .into(),
                );
            }
        }
        for stage in &d.stages {
            if create_decision_inputs::encode(&d, &stage.members, &stage.id)? != stage.input {
                return Err("Stored native input differs from its source snapshot".into());
            }
        }
        Ok(d)
    }
}
