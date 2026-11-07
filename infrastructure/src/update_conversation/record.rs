use crate::cache::json_string_len;
use crate::conversations;
use crate::conversations::StorageError;
use crate::create_tool_output::RESULT_TEXT_JSON_LIMIT;
use crate::instructions;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Record {
    version: u32,
    #[serde(default)]
    agent: Option<AgentRecord>,
    #[serde(default)]
    runs: Vec<RunRecord>,
    #[serde(default)]
    instruction_snapshots: Vec<SnapshotRecord>,
    #[serde(default)]
    selected_instructions: Option<String>,
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

#[derive(serde::Serialize, serde::Deserialize)]
struct AgentRecord {
    id: String,
    role: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct RunRecord {
    id: String,
    agent_id: String,
    reply_id: String,
    model_id: String,
    #[serde(default)]
    provider_id: Option<String>,
    effort: Option<EffortRecord>,
    status: String,
    #[serde(default)]
    instructions: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MessageRecord {
    id: String,
    role: String,
    content: String,
    status: String,
    model_id: String,
    #[serde(default)]
    provider_id: Option<String>,
    effort: Option<EffortRecord>,
    #[serde(default)]
    steps: Vec<StepRecord>,
    #[serde(default)]
    steering: Vec<SteeringRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    generation_usage: Option<Vec<Option<UsageRecord>>>,
}

impl From<domain::Conversation> for Record {
    fn from(conversation: domain::Conversation) -> Self {
        Self {
            version: 5,
            instruction_snapshots: conversation
                .instruction_snapshots
                .into_iter()
                .map(Into::into)
                .collect(),
            selected_instructions: conversation.selected_instructions,
            agent: conversation.agent.map(|agent| AgentRecord {
                id: agent.id.0,
                role: "griller".into(),
            }),
            runs: conversation
                .runs
                .into_iter()
                .map(|run| RunRecord {
                    id: run.id,
                    instructions: run.instructions,
                    agent_id: run.agent_id.0,
                    reply_id: run.reply_id,
                    model_id: run.model_id,
                    provider_id: Some(run.provider_id.0),
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
                    provider_id: Some(message.provider_id.0),
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

impl Record {
    pub(super) fn finishing_capacity(&self) -> usize {
        let integer_limit = u64::MAX.to_string().len();
        let run_capacity = self.runs.len().saturating_mul(8);
        let mut capacity = run_capacity + integer_limit - self.updated_at.to_string().len()
            + integer_limit
            - self.revision.to_string().len();
        let status_limit = [
            domain::ToolStatus::Succeeded,
            domain::ToolStatus::Failed,
            domain::ToolStatus::Invalid,
            domain::ToolStatus::Cancelled,
            domain::ToolStatus::Skipped,
            domain::ToolStatus::Interrupted,
            domain::ToolStatus::Unknown,
        ]
        .into_iter()
        .map(|status| json_string_len(status.as_str()))
        .max()
        .unwrap_or(0);
        for message in &self.messages {
            capacity = capacity.saturating_add(
                json_string_len("interrupted").saturating_sub(json_string_len(&message.status)),
            );
            for call in message.steps.iter().flat_map(|step| &step.calls) {
                if domain::ToolStatus::parse(&call.status).is_some_and(|status| !status.terminal())
                {
                    let current_text = json_string_len(&call.output)
                        .saturating_add(call.artifact.as_deref().map_or(4, json_string_len));
                    capacity = capacity
                        .saturating_add(RESULT_TEXT_JSON_LIMIT.saturating_sub(current_text))
                        .saturating_add(status_limit.saturating_sub(json_string_len(&call.status)))
                        .saturating_add(integer_limit - call.elapsed_ms.to_string().len())
                        .saturating_add(usize::from(call.truncated));
                }
            }
        }
        capacity
    }
}

impl TryFrom<Record> for domain::Conversation {
    type Error = StorageError;

    fn try_from(dto: Record) -> Result<Self, Self::Error> {
        if !matches!(dto.version, 1..=5) {
            return Err("Unsupported conversation format".into());
        }
        conversations::validate_id(&dto.id)?;
        conversations::validate_id(&dto.provider_connection_id)?;
        let mut messages = Vec::new();
        let mut ids = std::collections::HashSet::new();
        for (index, message) in dto.messages.into_iter().enumerate() {
            conversations::validate_id(&message.id)?;
            if dto.version == 5
                && message
                    .provider_id
                    .as_ref()
                    .is_none_or(|id| !matches!(id.as_str(), "openai" | "openrouter"))
            {
                return Err("Missing or invalid message provider provenance".into());
            }
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
                provider_id: domain::ProviderId(
                    message
                        .provider_id
                        .unwrap_or_else(|| dto.model.provider_id.clone()),
                ),
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
        if dto.agent.is_some() && dto.workspace.is_none() {
            return Err("Associated Griller must have a stored workspace".into());
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
        if dto.instruction_snapshots.len() > 128 {
            return Err("Instruction snapshot history exceeds its limit".into());
        }
        let instruction_snapshots = dto
            .instruction_snapshots
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<domain::InstructionSnapshot>, StorageError>>()?;
        let mut snapshot_ids = std::collections::HashSet::new();
        for snapshot in &instruction_snapshots {
            if !snapshot_ids.insert(&snapshot.id) {
                return Err("Duplicate instruction snapshot identity".into());
            }
        }
        if dto
            .selected_instructions
            .as_ref()
            .is_some_and(|id| !snapshot_ids.contains(id))
        {
            return Err("Selected instruction snapshot is missing".into());
        }
        let mut run_ids = std::collections::HashSet::new();
        let mut replies = std::collections::HashSet::new();
        let runs = dto
            .runs
            .into_iter()
            .enumerate()
            .map(|(index, run)| {
                conversations::validate_id(&run.id)?;
                conversations::validate_id(&run.agent_id)?;
                conversations::validate_id(&run.reply_id)?;
                let reply = messages
                    .get(index * 2 + 1)
                    .filter(|message| {
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
                    || dto.version == 5 && run.provider_id.is_none()
                    || run
                        .provider_id
                        .as_ref()
                        .is_some_and(|id| id != &reply.provider_id.0)
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
                if run
                    .instructions
                    .as_ref()
                    .is_some_and(|id| !snapshot_ids.contains(id))
                {
                    return Err(StorageError::from("Run instruction snapshot is missing"));
                }
                Ok(domain::Run {
                    instructions: run.instructions,
                    id: run.id,
                    agent_id: domain::AgentId(run.agent_id),
                    reply_id: run.reply_id,
                    model_id: run.model_id,
                    provider_id: reply.provider_id.clone(),
                    effort,
                    status,
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        if agent.is_some() && runs.len() != messages.len() / 2 {
            return Err("Griller messages must belong to recorded runs".into());
        }
        let model: domain::Model = dto.model.into();
        let effort = dto.effort.map(Into::into);
        model.validate_effort(effort)?;
        Ok(Self {
            id: dto.id,
            agent,
            runs,
            instruction_snapshots,
            selected_instructions: dto.selected_instructions,
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

#[derive(serde::Serialize, serde::Deserialize)]
struct SnapshotRecord {
    id: String,
    phase: String,
    scopes: Vec<String>,
    references: Vec<String>,
    instructions: Vec<InstructionRecord>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InstructionRecord {
    id: String,
    version: String,
    source: String,
    upstream: Vec<String>,
    requires: Vec<String>,
    body: String,
}

impl From<domain::InstructionSnapshot> for SnapshotRecord {
    fn from(snapshot: domain::InstructionSnapshot) -> Self {
        Self {
            id: snapshot.id,
            phase: snapshot.phase,
            scopes: snapshot.scopes,
            references: snapshot.references,
            instructions: snapshot
                .instructions
                .into_iter()
                .map(|instruction| InstructionRecord {
                    id: instruction.id,
                    version: instruction.version,
                    source: instruction.source,
                    upstream: instruction.upstream,
                    requires: instruction.requires,
                    body: instruction.body,
                })
                .collect(),
        }
    }
}

impl TryFrom<SnapshotRecord> for domain::InstructionSnapshot {
    type Error = StorageError;

    fn try_from(record: SnapshotRecord) -> Result<Self, Self::Error> {
        let snapshot = Self {
            id: record.id,
            phase: record.phase,
            scopes: record.scopes,
            references: record.references,
            instructions: record
                .instructions
                .into_iter()
                .map(|instruction| domain::Instruction {
                    id: instruction.id,
                    version: instruction.version,
                    source: instruction.source,
                    upstream: instruction.upstream,
                    requires: instruction.requires,
                    body: instruction.body,
                })
                .collect(),
        };
        instructions::validate(&snapshot)?;
        Ok(snapshot)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
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
    #[serde(default)]
    usage: Option<UsageRecord>,
    #[serde(default)]
    status: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct SteeringRecord {
    pub id: String,
    pub text: String,
    pub after_step: usize,
}
#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
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
impl TryFrom<StepRecord> for domain::ModelStep {
    type Error = StorageError;
    fn try_from(step: StepRecord) -> Result<Self, Self::Error> {
        conversations::validate_id(&step.id)?;
        if step
            .resolved_model_id
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 512 || s.chars().any(char::is_control))
            || step
                .catalog_canonical_id
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 512 || s.chars().any(char::is_control))
            || step
                .serving_provider
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 256 || s.chars().any(char::is_control))
            || step.output.len() > 1_048_576
        {
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
            resolved_model_id: step.resolved_model_id,
            catalog_canonical_id: step.catalog_canonical_id,
            serving_provider: step.serving_provider,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ModelRecord {
    id: String,
    name: String,
    #[serde(default)]
    context_window: Option<u64>,
    #[serde(default)]
    canonical_id: Option<String>,
    #[serde(default)]
    worker_supported: Option<bool>,
    provider_id: String,
    #[serde(default)]
    efforts: Vec<EffortRecord>,
    pricing: Option<PricingRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            worker_supported: model.worker_supported,
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

impl From<ModelRecord> for domain::Model {
    fn from(dto: ModelRecord) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            context_window: dto.context_window,
            canonical_id: dto.canonical_id,
            worker_supported: dto.worker_supported,
            provider_id: domain::ProviderId(dto.provider_id),
            efforts: dto.efforts.into_iter().map(Into::into).collect(),
            pricing: dto.pricing.map(Into::into),
        }
    }
}
