use super::Error;
use std::collections::BTreeMap;
use std::collections::HashSet;

pub(super) struct Ownership {
    pub parent: Option<String>,
    pub work: Option<String>,
    pub known_runs: HashSet<String>,
}

pub(super) struct ConversationSource {
    pub id: String,
    pub ownership: Ownership,
    pub entries: Vec<domain::UsageEntry>,
    pub unknown_history: Vec<String>,
}

pub(super) struct NativeSource {
    pub parent: String,
    pub run: Option<String>,
    pub entries: Vec<domain::UsageEntry>,
    pub unassigned: usize,
}

pub(super) fn ownership(
    conversation: &domain::Conversation,
    needed_runs: &HashSet<String>,
) -> Ownership {
    Ownership {
        parent: conversation
            .research
            .as_ref()
            .map(|research| research.parent_id.clone()),
        work: conversation
            .assignment
            .as_ref()
            .map(|assignment| assignment.work_id.clone()),
        known_runs: conversation
            .runs
            .iter()
            .map(|run| &run.id)
            .filter(|id| needed_runs.contains(*id))
            .cloned()
            .collect(),
    }
}

pub(super) fn conversation(
    conversation: domain::Conversation,
    run_id: Option<String>,
    needed_runs: HashSet<String>,
) -> Result<ConversationSource, Error> {
    let ownership = ownership(&conversation, &needed_runs);
    let mut entries = Vec::new();
    let mut unknown_history = Vec::new();
    for message in conversation
        .messages
        .iter()
        .filter(|message| message.role == domain::MessageRole::Assistant)
    {
        let run = conversation
            .runs
            .iter()
            .find(|run| run.reply_id == message.id);
        if run_id
            .as_ref()
            .is_some_and(|id| run.is_none_or(|run| &run.id != id))
        {
            continue;
        }
        if message.generation_usage.is_none()
            && (!message.steering.is_empty()
                || message.steps.is_empty() && message.status != domain::MessageStatus::InProgress
                || matches!(
                    message.status,
                    domain::MessageStatus::Cancelled
                        | domain::MessageStatus::Interrupted
                        | domain::MessageStatus::Failed
                ))
        {
            unknown_history.push(format!("{}:{}", conversation.id, message.id));
        }
        let legacy;
        let reports = if let Some(reports) = &message.generation_usage {
            reports.as_slice()
        } else {
            legacy = message
                .steps
                .iter()
                .map(|step| step.usage)
                .collect::<Vec<_>>();
            legacy.as_slice()
        };
        for (index, tokens) in reports.iter().enumerate() {
            let mut accounting = message
                .generation_accounting
                .as_ref()
                .and_then(|reports| reports.get(index))
                .cloned()
                .unwrap_or_else(domain::GenerationAccounting::unknown);
            if message.generation_accounting.is_none() && message.generation_usage.is_none() {
                accounting.reported_usd = (message.provider_id.0 == "openrouter")
                    .then(|| message.steps.get(index).and_then(|step| step.reported_cost))
                    .flatten();
            }
            entries.push(domain::UsageEntry {
                id: if message.generation_usage.is_some() {
                    format!("worker:{}:{}:{index}", conversation.id, message.id)
                } else {
                    format!(
                        "worker:{}:{}:step:{}",
                        conversation.id, message.id, message.steps[index].id
                    )
                },
                source_id: conversation.id.clone(),
                source_revision: conversation.revision,
                agent_id: conversation.agent.as_ref().map(|agent| agent.id.0.clone()),
                run_id: run.map(|run| run.id.clone()),
                work_id: None,
                category: "worker".into(),
                provider_id: message.provider_id.0.clone(),
                connection_id: accounting.connection_id.clone(),
                model_id: message.model_id.clone(),
                billing: if message.provider_id.0 == "openai" {
                    "subscription_allowance"
                } else {
                    "api_currency"
                }
                .into(),
                admission: if tokens.is_some() {
                    "reported"
                } else {
                    "reserved_or_sent_unknown"
                }
                .into(),
                tokens: *tokens,
                accounting,
            });
            if entries.len().saturating_add(unknown_history.len()) > 100_000 {
                return Err(Error::blocked(
                    "Usage entry bound reached; required facts cannot be omitted",
                ));
            }
        }
        if unknown_history.len() > 100_000 {
            return Err(Error::blocked(
                "Historical usage uncertainty exceeds its representation bound",
            ));
        }
    }
    Ok(ConversationSource {
        id: conversation.id,
        ownership,
        entries,
        unknown_history,
    })
}

pub(super) fn native(decision: domain::AssignmentDecision) -> Option<NativeSource> {
    let request = decision.request.as_ref()?;
    let reports = if decision.legacy {
        vec![(
            format!("native:{}:legacy", decision.id),
            decision.response.as_ref(),
            "legacy_request_unknown",
        )]
    } else {
        decision
            .stages
            .iter()
            .filter(|stage| stage.started_at.is_some())
            .map(|stage| {
                (
                    format!("native:{}:{}", decision.id, stage.id),
                    stage.response.as_ref(),
                    stage.status(),
                )
            })
            .collect()
    };
    let entries: Vec<_> = reports
        .into_iter()
        .map(|(id, report, status)| domain::UsageEntry {
            id,
            source_id: decision.id.clone(),
            source_revision: decision.revision,
            agent_id: Some(decision.parent_agent_id.clone()),
            run_id: decision.parent_run_id.clone(),
            work_id: Some(decision.work_id.clone()),
            category: "native_routing".into(),
            provider_id: request.model.provider_id.0.clone(),
            connection_id: Some(request.connection_id.clone()),
            model_id: request.model.id.clone(),
            billing: "api_currency".into(),
            admission: if report.is_some() {
                "reported"
            } else {
                "reserved_or_sent_unknown"
            }
            .into(),
            tokens: report.map(|report| domain::TokenUsage {
                input_tokens: report.input_tokens,
                output_tokens: report.output_tokens,
            }),
            accounting: domain::GenerationAccounting {
                connection_id: Some(request.connection_id.clone()),
                outcome: status.into(),
                reported_usd: report.and_then(|report| report.cost),
                estimated_usd: None,
                pricing_source: request
                    .model
                    .pricing
                    .as_ref()
                    .map(|_| "catalog_static_usd_per_million_tokens".into()),
                pricing: request.model.pricing.clone(),
            },
        })
        .collect();
    let unassigned = if decision.parent_run_id.is_none() {
        entries.len()
    } else {
        0
    };
    Some(NativeSource {
        parent: decision.parent_id,
        run: decision.parent_run_id,
        entries,
        unassigned,
    })
}

pub(super) fn work(
    ownership: &BTreeMap<String, Ownership>,
    id: &str,
) -> Result<Option<String>, Error> {
    let mut id = id;
    let mut seen = HashSet::new();
    loop {
        if seen.len() >= 256 || !seen.insert(id) {
            return Err(Error::blocked(
                "Usage ownership is cyclic or exceeds its bound",
            ));
        }
        let source = ownership
            .get(id)
            .ok_or_else(|| Error::blocked("Usage ownership source is unavailable"))?;
        if let Some(work) = &source.work {
            return Ok(Some(work.clone()));
        }
        let Some(parent) = &source.parent else {
            return Ok(None);
        };
        id = parent;
    }
}
