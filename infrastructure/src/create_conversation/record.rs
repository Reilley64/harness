#[derive(serde::Serialize)]
pub(super) struct Record {
    version: u32,
    agent: Option<AgentRecord>,
    runs: Vec<RunRecord>,
    resume_required: bool,
    workspace: Option<std::path::PathBuf>,
    id: String,
    provider_connection_id: String,
    model: ModelRecord,
    effort: Option<EffortRecord>,
    messages: Vec<MessageRecord>,
    created_at: u64,
    updated_at: u64,
    revision: u64,
}

#[derive(serde::Serialize)]
struct AgentRecord {
    id: String,
    role: String,
}
#[derive(serde::Serialize)]
struct RunRecord {
    id: String,
    agent_id: String,
    reply_id: String,
    model_id: String,
    provider_id: String,
    effort: Option<EffortRecord>,
    status: String,
}

#[derive(serde::Serialize)]
struct MessageRecord {
    id: String,
    role: String,
    content: String,
    status: String,
    model_id: String,
    provider_id: String,
    effort: Option<EffortRecord>,
    steps: Vec<StepRecord>,
    steering: Vec<SteeringRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_usage: Option<Vec<Option<UsageRecord>>>,
}

impl From<domain::Conversation> for Record {
    fn from(conversation: domain::Conversation) -> Self {
        Self {
            version: 5,
            agent: conversation.agent.map(|agent| AgentRecord {
                id: agent.id.0,
                role: "griller".into(),
            }),
            runs: conversation
                .runs
                .into_iter()
                .map(|run| RunRecord {
                    id: run.id,
                    agent_id: run.agent_id.0,
                    reply_id: run.reply_id,
                    model_id: run.model_id,
                    provider_id: run.provider_id.0,
                    effort: run.effort.map(Into::into),
                    status: match run.status {
                        domain::RunStatus::Running => "running",
                        domain::RunStatus::NeedsHuman => "needs_human",
                        domain::RunStatus::Stopped => "stopped",
                        domain::RunStatus::Failed => "failed",
                        domain::RunStatus::Interrupted => "interrupted",
                    }
                    .into(),
                })
                .collect(),
            resume_required: conversation.resume_required,
            workspace: conversation.workspace,
            id: conversation.id,
            provider_connection_id: conversation.provider_connection_id,
            model: conversation.model.into(),
            effort: conversation.effort.map(Into::into),
            messages: conversation
                .messages
                .into_iter()
                .map(|message| MessageRecord {
                    id: message.id,
                    role: match message.role {
                        domain::MessageRole::User => "user",
                        domain::MessageRole::Assistant => "assistant",
                    }
                    .into(),
                    content: message.content,
                    status: match message.status {
                        domain::MessageStatus::InProgress => "in_progress",
                        domain::MessageStatus::Completed => "completed",
                        domain::MessageStatus::Cancelled => "cancelled",
                        domain::MessageStatus::Failed => "failed",
                        domain::MessageStatus::Interrupted => "interrupted",
                        domain::MessageStatus::Incomplete => "incomplete",
                        domain::MessageStatus::Limited => "limited",
                        domain::MessageStatus::Refused => "refused",
                    }
                    .into(),
                    model_id: message.model_id,
                    provider_id: message.provider_id.0,
                    effort: message.effort.map(Into::into),
                    generation_usage: message.generation_usage.map(|reports| {
                        reports
                            .into_iter()
                            .map(|report| {
                                report.map(|usage| UsageRecord {
                                    input_tokens: usage.input_tokens,
                                    output_tokens: usage.output_tokens,
                                })
                            })
                            .collect()
                    }),
                    steps: message.steps.into_iter().map(Into::into).collect(),
                    steering: message
                        .steering
                        .into_iter()
                        .map(|s| SteeringRecord {
                            id: s.id,
                            text: s.text,
                            after_step: s.after_step,
                        })
                        .collect(),
                })
                .collect(),
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
            revision: conversation.revision,
        }
    }
}

#[derive(serde::Serialize)]
struct StepRecord {
    id: String,
    output: String,
    #[serde(default)]
    resolved_model_id: Option<String>,
    #[serde(default)]
    catalog_canonical_id: Option<String>,
    #[serde(default)]
    serving_provider: Option<String>,
    calls: Vec<CallRecord>,
    usage: Option<UsageRecord>,
    status: Option<String>,
}
#[derive(serde::Serialize)]
struct SteeringRecord {
    pub id: String,
    pub text: String,
    pub after_step: usize,
}
#[derive(serde::Serialize)]
struct CallRecord {
    id: String,
    call_id: String,
    name: String,
    arguments: String,
    status: String,
    output: String,
    artifact: Option<String>,
    truncated: bool,
    elapsed_ms: u64,
    text_offset: usize,
}

#[derive(serde::Serialize)]
struct UsageRecord {
    input_tokens: u64,
    output_tokens: u64,
}

impl From<domain::ModelStep> for StepRecord {
    fn from(step: domain::ModelStep) -> Self {
        Self {
            id: step.id,
            output: step.output,
            resolved_model_id: step.resolved_model_id,
            catalog_canonical_id: step.catalog_canonical_id,
            serving_provider: step.serving_provider,
            usage: step.usage.map(|usage| UsageRecord {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
            }),
            status: Some(
                match step.status {
                    domain::MessageStatus::Completed => "completed",
                    domain::MessageStatus::Cancelled => "cancelled",
                    domain::MessageStatus::Refused => "refused",
                    domain::MessageStatus::Incomplete => "incomplete",
                    _ => "failed",
                }
                .into(),
            ),
            calls: step
                .calls
                .into_iter()
                .map(|c| CallRecord {
                    id: c.id,
                    call_id: c.call_id,
                    name: c.name,
                    arguments: c.arguments,
                    status: c.status.as_str().into(),
                    output: c.output,
                    artifact: c.artifact,
                    truncated: c.truncated,
                    elapsed_ms: c.elapsed_ms,
                    text_offset: c.text_offset,
                })
                .collect(),
        }
    }
}

#[derive(serde::Serialize)]
struct ModelRecord {
    id: String,
    name: String,
    context_window: Option<u64>,
    #[serde(default)]
    canonical_id: Option<String>,
    #[serde(default)]
    capabilities: Vec<CapabilityRecord>,
    provider_id: String,
    efforts: Vec<EffortRecord>,
    pricing: Option<PricingRecord>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum EffortRecord {
    Low,
    Medium,
    High,
    ExtraHigh,
}

#[derive(serde::Serialize)]
struct PricingRecord {
    input: Option<f64>,
    cached_input: Option<f64>,
    output: Option<f64>,
}

impl From<domain::ModelEffort> for EffortRecord {
    fn from(effort: domain::ModelEffort) -> Self {
        match effort {
            domain::ModelEffort::Low => Self::Low,
            domain::ModelEffort::Medium => Self::Medium,
            domain::ModelEffort::High => Self::High,
            domain::ModelEffort::ExtraHigh => Self::ExtraHigh,
        }
    }
}

impl From<domain::Model> for ModelRecord {
    fn from(model: domain::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            context_window: model.context_window,
            canonical_id: model.canonical_id,
            capabilities: model.capabilities.into_iter().map(Into::into).collect(),
            provider_id: model.provider_id.0,
            efforts: model.efforts.into_iter().map(Into::into).collect(),
            pricing: model.pricing.map(|pricing| PricingRecord {
                input: pricing.input,
                cached_input: pricing.cached_input,
                output: pricing.output,
            }),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum CapabilityRecord {
    Chat,
    Decisions,
}

impl From<domain::ModelCapability> for CapabilityRecord {
    fn from(capability: domain::ModelCapability) -> Self {
        match capability {
            domain::ModelCapability::Chat => Self::Chat,
            domain::ModelCapability::Decisions => Self::Decisions,
        }
    }
}
