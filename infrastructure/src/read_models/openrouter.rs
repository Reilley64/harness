#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Response {
    data: Vec<Model>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Model {
    id: String,
    canonical_slug: Option<String>,
    name: String,
    context_length: Option<u64>,
    architecture: Option<Architecture>,
    supported_parameters: Option<Vec<String>>,
    endpoints: Option<Vec<String>>,
    reasoning: Option<serde_json::Value>,
    pricing: Option<Pricing>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Architecture {
    input_modalities: Option<Vec<String>>,
    output_modalities: Option<Vec<String>>,
    tokenizer: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Pricing {
    prompt: Option<String>,
    completion: Option<String>,
    input_cache_read: Option<String>,
}

fn price(value: Option<String>) -> Option<f64> {
    value
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .map(|v| v * 1_000_000.0)
        .filter(|v| v.is_finite())
}

impl Response {
    pub(super) fn models(self) -> Vec<domain::Model> {
        self.data
            .into_iter()
            .map(|model| {
                let parameters = model.supported_parameters.as_deref().unwrap_or_default();
                let architecture = model.architecture.as_ref();
                let worker_supported = Some(
                    architecture.is_some_and(|a| {
                        a.input_modalities
                            .as_ref()
                            .is_some_and(|v| v.iter().any(|s| s == "text"))
                            && a.output_modalities.as_ref().is_some_and(|v| v == &["text"])
                            && a.tokenizer
                                .as_ref()
                                .is_some_and(|s| !s.is_empty() && !s.eq_ignore_ascii_case("router"))
                    }) && model
                        .endpoints
                        .as_ref()
                        .is_some_and(|v| v.iter().any(|s| s == "/api/v1/responses"))
                        && parameters.iter().any(|s| s == "tools")
                        && model
                            .context_length
                            .is_some_and(|n| n > 0 && n <= 1_000_000_000),
                );
                let mut efforts = Vec::new();
                if parameters.iter().any(|s| s == "reasoning")
                    && let Some(reasoning) = model.reasoning
                    && let Some(supported) = reasoning.get("supported_efforts")
                {
                    for (name, effort) in [
                        ("low", domain::ModelEffort::Low),
                        ("medium", domain::ModelEffort::Medium),
                        ("high", domain::ModelEffort::High),
                        ("xhigh", domain::ModelEffort::ExtraHigh),
                    ] {
                        if supported.is_null()
                            || supported
                                .as_array()
                                .is_some_and(|v| v.iter().any(|s| s.as_str() == Some(name)))
                        {
                            efforts.push(effort);
                        }
                    }
                }
                domain::Model {
                    id: model.id,
                    canonical_id: model.canonical_slug,
                    name: model.name,
                    context_window: model.context_length,
                    worker_supported,
                    provider_id: domain::ProviderId("openrouter".into()),
                    efforts,
                    pricing: model.pricing.map(|p| domain::ModelPricing {
                        input: price(p.prompt),
                        cached_input: price(p.input_cache_read),
                        output: price(p.completion),
                    }),
                }
            })
            .collect()
    }
}
