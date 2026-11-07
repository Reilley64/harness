#[derive(serde::Serialize)]
pub(super) struct Record {
    version: u32,
    agent: AgentRecord,
    runs: [(); 0],
    resume_required: bool,
    workspace: std::path::PathBuf,
    id: String,
    provider_connection_id: String,
    model: ModelRecord,
    effort: Option<&'static str>,
    messages: [(); 0],
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
struct ModelRecord {
    id: String,
    name: String,
    context_window: Option<u64>,
    provider_id: String,
    efforts: Vec<&'static str>,
    pricing: Option<PricingRecord>,
}

#[derive(serde::Serialize)]
struct PricingRecord {
    input: Option<f64>,
    cached_input: Option<f64>,
    output: Option<f64>,
}

impl Record {
    pub(super) fn new(
        id: String,
        agent_id: String,
        model: domain::Model,
        effort: Option<domain::ModelEffort>,
        provider_connection_id: String,
        workspace: std::path::PathBuf,
        now: u64,
        role: domain::AgentRole,
    ) -> Self {
        Self {
            version: if role == domain::AgentRole::Setup {
                5
            } else {
                3
            },
            agent: AgentRecord {
                id: agent_id,
                role: if role == domain::AgentRole::Setup {
                    "setup"
                } else {
                    "griller"
                },
            },
            runs: [],
            resume_required: false,
            workspace,
            id,
            provider_connection_id,
            model: ModelRecord {
                id: model.id,
                name: model.name,
                context_window: model.context_window,
                provider_id: model.provider_id.0,
                efforts: model.efforts.into_iter().map(effort_name).collect(),
                pricing: model.pricing.map(|pricing| PricingRecord {
                    input: pricing.input,
                    cached_input: pricing.cached_input,
                    output: pricing.output,
                }),
            },
            effort: effort.map(effort_name),
            messages: [],
            created_at: now,
            updated_at: now,
            revision: 0,
        }
    }
}

fn effort_name(effort: domain::ModelEffort) -> &'static str {
    match effort {
        domain::ModelEffort::Low => "low",
        domain::ModelEffort::Medium => "medium",
        domain::ModelEffort::High => "high",
        domain::ModelEffort::ExtraHigh => "extra_high",
    }
}
