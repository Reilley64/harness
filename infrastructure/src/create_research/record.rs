#[derive(serde::Serialize)]
pub(super) struct Record {
    version: u32,
    id: String,
    agent: AgentRecord,
    research: ResearchRecord,

    workspace: std::path::PathBuf,
    provider_connection_id: String,
    model: ModelRecord,
    effort: Option<String>,

    instruction_snapshots: Vec<SnapshotRecord>,
    selected_instructions: String,

    messages: Vec<()>,
    runs: Vec<()>,
    resume_required: bool,

    created_at: u64,
    updated_at: u64,
    revision: u64,
}

#[derive(serde::Serialize)]
struct AgentRecord {
    id: String,
    role: &'static str,
}

#[derive(serde::Serialize)]
struct ResearchRecord {
    parent_id: String,
    intent: String,

    assignment_id: String,
    execution_id: String,
}

#[derive(serde::Serialize)]
struct ModelRecord {
    id: String,
    name: String,
    provider_id: String,
    canonical_id: Option<String>,

    context_window: Option<u64>,
    capabilities: Vec<&'static str>,
    efforts: Vec<String>,
    pricing: Option<PricingRecord>,
}

#[derive(serde::Serialize)]
struct PricingRecord {
    input: Option<f64>,
    cached_input: Option<f64>,
    output: Option<f64>,
}

#[derive(serde::Serialize)]
struct SnapshotRecord {
    id: String,
    phase: String,

    scopes: Vec<String>,
    references: Vec<String>,
    instructions: Vec<InstructionRecord>,
}

#[derive(serde::Serialize)]
struct InstructionRecord {
    id: String,
    version: String,
    source: String,
    upstream: Vec<String>,

    requires: Vec<String>,
    body: String,
}

fn effort(value: domain::ModelEffort) -> String {
    match value {
        domain::ModelEffort::Low => "low",
        domain::ModelEffort::Medium => "medium",
        domain::ModelEffort::High => "high",
        domain::ModelEffort::ExtraHigh => "extra_high",
    }
    .into()
}

impl Record {
    pub(super) fn new(
        execution: domain::ChildExecution,
        workspace: std::path::PathBuf,
        now: u64,
    ) -> Self {
        let snapshot = execution.instructions;
        let model = execution.selected.model;
        Self {
            version: 7,
            id: execution.agent_id.clone(),
            agent: AgentRecord {
                id: execution.agent_id,
                role: "research",
            },
            research: ResearchRecord {
                parent_id: execution.parent_id,
                intent: execution.intent,
                assignment_id: execution.assignment_id,
                execution_id: execution.execution_id,
            },
            workspace,
            provider_connection_id: execution.selected.connection_id,
            model: ModelRecord {
                id: model.id,
                name: model.name,
                provider_id: model.provider_id.0,
                canonical_id: model.canonical_id,
                context_window: model.context_window,
                capabilities: model
                    .capabilities
                    .into_iter()
                    .map(|capability| match capability {
                        domain::ModelCapability::Chat => "chat",
                        domain::ModelCapability::Decisions => "decisions",
                    })
                    .collect(),
                efforts: model.efforts.into_iter().map(effort).collect(),
                pricing: model.pricing.map(|pricing| PricingRecord {
                    input: pricing.input,
                    cached_input: pricing.cached_input,
                    output: pricing.output,
                }),
            },
            effort: execution.selected.selection.effort.map(effort),
            selected_instructions: snapshot.id.clone(),
            instruction_snapshots: vec![SnapshotRecord {
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
            }],
            messages: Vec::new(),
            runs: Vec::new(),
            resume_required: false,
            created_at: now,
            updated_at: now,
            revision: 0,
        }
    }
}
