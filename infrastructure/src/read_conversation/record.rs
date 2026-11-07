use crate::conversations;
use crate::conversations::StorageError;

#[derive(serde::Deserialize)]
pub(super) struct Record {
    version: u32,
    #[serde(default)]
    agent: Option<AgentRecord>,
    #[serde(default)]
    runs: Vec<RunRecord>,
    #[serde(default)]
    resume_required: bool,
    #[serde(default)]
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

#[derive(serde::Deserialize)]
struct AgentRecord {
    id: String,
    role: String,
}
#[derive(serde::Deserialize)]
struct RunRecord {
    id: String,
    agent_id: String,
    reply_id: String,
    model_id: String,
    effort: Option<EffortRecord>,
    status: String,
}

#[derive(serde::Deserialize)]
struct MessageRecord {
    id: String,
    role: String,
    content: String,
    status: String,
    model_id: String,
    effort: Option<EffortRecord>,
    #[serde(default)]
    steps: Vec<StepRecord>,
    #[serde(default)]
    steering: Vec<SteeringRecord>,
    #[serde(default)]
    generation_usage: Option<Vec<Option<UsageRecord>>>,
}

impl TryFrom<Record> for domain::Conversation {
    type Error = StorageError;

    fn try_from(dto: Record) -> Result<Self, Self::Error> {
        if !matches!(dto.version, 1..=3) {
            return Err("Unsupported conversation format".into());
        }
        conversations::validate_id(&dto.id)?;
        conversations::validate_id(&dto.provider_connection_id)?;
        let mut messages = Vec::new();
        let mut ids = std::collections::HashSet::new();
        for (index, message) in dto.messages.into_iter().enumerate() {
            conversations::validate_id(&message.id)?;
            if !ids.insert(message.id.clone()) {
                return Err("Duplicate message identity".into());
            }
            let role = match message.role.as_str() {
                "user" if index % 2 == 0 => domain::MessageRole::User,
                "assistant" if index % 2 == 1 => domain::MessageRole::Assistant,
                _ => return Err("Invalid conversation message order".into()),
            };
            let status = match message.status.as_str() {
                "in_progress" => domain::MessageStatus::InProgress,
                "completed" => domain::MessageStatus::Completed,
                "cancelled" => domain::MessageStatus::Cancelled,
                "failed" => domain::MessageStatus::Failed,
                "interrupted" => domain::MessageStatus::Interrupted,
                "incomplete" => domain::MessageStatus::Incomplete,
                "limited" => domain::MessageStatus::Limited,
                "refused" => domain::MessageStatus::Refused,
                _ => return Err("Invalid message status".into()),
            };
            if role == domain::MessageRole::User && status != domain::MessageStatus::Completed {
                return Err("Invalid user message status".into());
            }
            messages.push(domain::Message {
                id: message.id,
                role,
                content: message.content,
                status,
                model_id: message.model_id,
                effort: message.effort.map(Into::into),
                generation_usage: message
                    .generation_usage
                    .map(|reports| {
                        reports
                            .into_iter()
                            .map(|report| {
                                report
                                    .map(|usage| {
                                        domain::TokenUsage::new(
                                            usage.input_tokens,
                                            usage.output_tokens,
                                        )
                                    })
                                    .transpose()
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?,
                steps: message
                    .steps
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                steering: message
                    .steering
                    .into_iter()
                    .map(|s| domain::SteeringInstruction {
                        id: s.id,
                        text: s.text,
                        after_step: s.after_step,
                    })
                    .collect(),
            });
        }
        let mut calls = std::collections::HashSet::new();
        let mut provider_calls = std::collections::HashSet::new();
        for message in &messages {
            if message.content.len() > 1_048_576
                || message.steps.len() > domain::MAX_MODEL_STEPS
                || message.steering.len() > 64
                || message.generation_usage.as_ref().is_some_and(|reports| {
                    reports.len() > domain::MAX_MODEL_STEPS || reports.len() < message.steps.len()
                })
                || message.steps.iter().map(|s| s.output.len()).sum::<usize>() > 4_194_304
            {
                return Err("Conversation turn exceeds its limits".into());
            }
            if message.role == domain::MessageRole::User
                && (!message.steps.is_empty()
                    || !message.steering.is_empty()
                    || message.generation_usage.is_some())
            {
                return Err("User messages cannot contain tool executions".into());
            }
            if let Some(reports) = &message.generation_usage {
                let mut reports = reports.iter();
                for step in &message.steps {
                    if !reports.any(|report| *report == step.usage) {
                        return Err("Model-step usage does not match generation accounting".into());
                    }
                }
            }
            for call in message.steps.iter().flat_map(|s| &s.calls) {
                if !calls.insert(&call.id)
                    || !provider_calls.insert(&call.call_id)
                    || call.text_offset > message.content.len()
                    || !message.content.is_char_boundary(call.text_offset)
                {
                    return Err("Invalid tool identity or text position".into());
                }
            }
            let mut instructions = std::collections::HashSet::new();
            for instruction in &message.steering {
                conversations::validate_id(&instruction.id)?;
                if !instructions.insert(&instruction.id)
                    || instruction.text.len() > 1_048_576
                    || instruction.after_step > message.steps.len()
                {
                    return Err("Invalid steering record".into());
                }
            }
        }
        if dto.workspace.as_ref().is_some_and(|p| !p.is_absolute()) {
            return Err("Stored workspace must be absolute".into());
        }
        if messages.len() % 2 != 0 {
            return Err("Incomplete conversation record".into());
        }
        let agent = dto
            .agent
            .map(|agent| {
                conversations::validate_id(&agent.id)?;
                if agent.role != "griller" {
                    return Err(StorageError::from("Unsupported agent role"));
                }
                Ok(domain::Agent {
                    id: domain::AgentId(agent.id),
                    role: domain::AgentRole::Griller,
                })
            })
            .transpose()?;
        let mut run_ids = std::collections::HashSet::new();
        let mut replies = std::collections::HashSet::new();
        let runs = dto
            .runs
            .into_iter()
            .map(|run| {
                conversations::validate_id(&run.id)?;
                conversations::validate_id(&run.agent_id)?;
                conversations::validate_id(&run.reply_id)?;
                let reply = messages
                    .iter()
                    .find(|message| {
                        message.id == run.reply_id && message.role == domain::MessageRole::Assistant
                    })
                    .ok_or("Run reply was not found")?;
                let effort = run.effort.map(Into::into);
                if !run_ids.insert(run.id.clone())
                    || !replies.insert(run.reply_id.clone())
                    || agent
                        .as_ref()
                        .is_none_or(|agent| agent.id.0 != run.agent_id)
                    || run.model_id != reply.model_id
                    || effort != reply.effort
                {
                    return Err(StorageError::from("Invalid run association"));
                }
                let status = match run.status.as_str() {
                    "running" if reply.status == domain::MessageStatus::InProgress => {
                        domain::RunStatus::Running
                    }
                    "needs_human"
                        if matches!(
                            reply.status,
                            domain::MessageStatus::Completed
                                | domain::MessageStatus::Limited
                                | domain::MessageStatus::Incomplete
                                | domain::MessageStatus::Refused
                        ) =>
                    {
                        domain::RunStatus::NeedsHuman
                    }
                    "stopped" if reply.status == domain::MessageStatus::Cancelled => {
                        domain::RunStatus::Stopped
                    }
                    "failed" if reply.status == domain::MessageStatus::Failed => {
                        domain::RunStatus::Failed
                    }
                    "interrupted" if reply.status == domain::MessageStatus::Interrupted => {
                        domain::RunStatus::Interrupted
                    }
                    _ => return Err("Invalid run status".into()),
                };
                Ok(domain::Run {
                    id: run.id,
                    agent_id: domain::AgentId(run.agent_id),
                    reply_id: run.reply_id,
                    model_id: run.model_id,
                    effort,
                    status,
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        let model: domain::Model = dto.model.into();
        let effort = dto.effort.map(Into::into);
        model.validate_effort(effort)?;
        Ok(Self {
            id: dto.id,
            agent,
            runs,
            resume_required: dto.resume_required,
            provider_connection_id: dto.provider_connection_id,
            workspace: dto.workspace,
            model,
            effort,
            messages,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            revision: dto.revision,
        })
    }
}

#[derive(serde::Deserialize)]
struct StepRecord {
    id: String,
    output: String,
    calls: Vec<CallRecord>,
    #[serde(default)]
    usage: Option<UsageRecord>,
    #[serde(default)]
    status: Option<String>,
}
#[derive(serde::Deserialize)]
struct SteeringRecord {
    pub id: String,
    pub text: String,
    pub after_step: usize,
}
#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
struct UsageRecord {
    input_tokens: u64,
    output_tokens: u64,
}

impl TryFrom<StepRecord> for domain::ModelStep {
    type Error = StorageError;
    fn try_from(step: StepRecord) -> Result<Self, Self::Error> {
        conversations::validate_id(&step.id)?;
        if step.output.len() > 1_048_576 {
            return Err("Invalid model-step continuation".into());
        }
        let parsed: serde_json::Value = serde_json::from_str(&step.output)?;
        if !parsed.is_array() {
            return Err("Invalid model-step continuation".into());
        }
        let provider_calls = parsed
            .as_array()
            .ok_or("Invalid provider output")?
            .iter()
            .filter(|item| item["type"] == "function_call")
            .collect::<Vec<_>>();
        if provider_calls.len() != step.calls.len() || step.calls.len() > 16 {
            return Err("Provider continuation and tool records do not match".into());
        }
        for (provider, call) in provider_calls.iter().zip(&step.calls) {
            if provider["call_id"] != call.call_id
                || provider["name"] != call.name
                || provider["arguments"] != call.arguments
            {
                return Err("Provider tool record was modified".into());
            }
        }
        let mut ids = std::collections::HashSet::new();
        let mut calls = Vec::new();
        for c in step.calls {
            conversations::validate_id(&c.id)?;
            if !ids.insert(c.call_id.clone())
                || c.call_id.is_empty()
                || c.call_id.len() > 256
                || c.name.len() > 256
                || c.arguments.len() > 65536
                || c.output.len() > 65536
            {
                return Err("Invalid tool call record".into());
            }
            calls.push(domain::ToolCall {
                id: c.id,
                call_id: c.call_id,
                name: c.name,
                arguments: c.arguments,
                status: domain::ToolStatus::parse(&c.status).ok_or("Invalid tool status")?,
                output: c.output,
                artifact: c.artifact,
                truncated: c.truncated,
                elapsed_ms: c.elapsed_ms,
                text_offset: c.text_offset,
            });
        }
        Ok(Self {
            id: step.id,
            output: step.output,
            usage: step
                .usage
                .map(|usage| domain::TokenUsage::new(usage.input_tokens, usage.output_tokens))
                .transpose()?,
            calls,
            status: match step.status.as_deref().unwrap_or("completed") {
                "completed" => domain::MessageStatus::Completed,
                "cancelled" => domain::MessageStatus::Cancelled,
                "refused" => domain::MessageStatus::Refused,
                "incomplete" => domain::MessageStatus::Incomplete,
                "failed" => domain::MessageStatus::Failed,
                _ => return Err("Invalid step status".into()),
            },
        })
    }
}

#[derive(serde::Deserialize)]
struct ModelRecord {
    id: String,
    name: String,
    #[serde(default)]
    context_window: Option<u64>,
    provider_id: String,
    #[serde(default)]
    efforts: Vec<EffortRecord>,
    pricing: Option<PricingRecord>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum EffortRecord {
    Low,
    Medium,
    High,
    ExtraHigh,
}

impl From<EffortRecord> for domain::ModelEffort {
    fn from(effort: EffortRecord) -> Self {
        match effort {
            EffortRecord::Low => Self::Low,
            EffortRecord::Medium => Self::Medium,
            EffortRecord::High => Self::High,
            EffortRecord::ExtraHigh => Self::ExtraHigh,
        }
    }
}

#[derive(serde::Deserialize)]
struct PricingRecord {
    input: Option<f64>,
    cached_input: Option<f64>,
    output: Option<f64>,
}

impl From<PricingRecord> for domain::ModelPricing {
    fn from(pricing: PricingRecord) -> Self {
        Self {
            input: pricing.input,
            cached_input: pricing.cached_input,
            output: pricing.output,
        }
    }
}

impl From<ModelRecord> for domain::Model {
    fn from(dto: ModelRecord) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            context_window: dto.context_window,
            provider_id: domain::ProviderId(dto.provider_id),
            efforts: dto.efforts.into_iter().map(Into::into).collect(),
            pricing: dto.pricing.map(Into::into),
        }
    }
}
