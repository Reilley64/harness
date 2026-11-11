use crate::conversations;
use agent_client_protocol::Error;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcRequest,
)]
#[request(method = "_harness/usage/read", response = Response)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request {
    pub session_id: Option<String>,
    pub run_id: Option<String>,
    pub work_id: Option<String>,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, agent_client_protocol::JsonRpcResponse,
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    pub entries: Vec<serde_json::Value>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub unknown_token_attempts: usize,
    pub reported_usd: Option<f64>,
    pub estimated_usd: Option<f64>,
    pub unknown_api_cost_attempts: usize,
    pub subscription_attempts: usize,
    pub unassigned_native_runs: usize,
    pub unknown_history_gaps: usize,
    pub unknown_history_sources: Vec<String>,
}

pub(crate) async fn response(
    output: application::use_cases::read_usage::Output,
) -> Result<Response, Error> {
    tokio::task::spawn_blocking(move || summary(output))
        .await
        .map_err(conversations::error)?
}

fn summary(output: application::use_cases::read_usage::Output) -> Result<Response, Error> {
    let has_history_gap = !output.unknown_history_sources.is_empty();
    let mut input = 0u64;
    let mut tokens_out = 0u64;
    let mut known_tokens = false;
    let mut unknown_tokens = 0;
    let mut reported = None;
    let mut estimated = None;
    let mut unknown_costs = 0;
    let mut subscriptions = 0;
    for entry in &output.entries {
        if let Some(tokens) = entry.tokens {
            known_tokens = true;
            input = input.saturating_add(tokens.input_tokens);
            tokens_out = tokens_out.saturating_add(tokens.output_tokens);
        } else {
            unknown_tokens += 1
        }
        if entry.billing == "subscription_allowance" {
            subscriptions += 1;
            continue;
        }
        if let Some(value) = entry.accounting.reported_usd {
            reported = Some(reported.unwrap_or(0.0) + value)
        } else {
            unknown_costs += 1
        }
        if let Some(value) = entry.accounting.estimated_usd {
            estimated = Some(estimated.unwrap_or(0.0) + value)
        }
    }
    if reported.is_some_and(|value: f64| !value.is_finite())
        || estimated.is_some_and(|value: f64| !value.is_finite())
    {
        return Err(Error::internal_error().data("Usage currency aggregate exceeds finite representation; inspect retained source reports. No amount was truncated."));
    }
    Ok(Response {
        entries: output.entries.into_iter().map(|entry| serde_json::json!({
            "id":entry.id, "sourceId":entry.source_id, "sourceRevision":entry.source_revision,
            "agentId":entry.agent_id,"runId":entry.run_id,"workId":entry.work_id,"category":entry.category,
            "providerId":entry.provider_id,"connectionId":entry.connection_id,"modelId":entry.model_id,
            "billing":entry.billing,"admission":entry.admission,"outcome":entry.accounting.outcome,
            "inputTokens":entry.tokens.map(|tokens|tokens.input_tokens),"outputTokens":entry.tokens.map(|tokens|tokens.output_tokens),
            "reportedUsd":entry.accounting.reported_usd,"estimatedUsd":entry.accounting.estimated_usd,
            "pricingSource":entry.accounting.pricing_source,
            "pricing":entry.accounting.pricing.as_ref().map(|pricing| serde_json::json!({"inputUsdPerMillion":pricing.input,"cachedInputUsdPerMillion":pricing.cached_input,"outputUsdPerMillion":pricing.output})),
            "pricingRetrievedAt":null,"pricingEffectiveAt":null,
        })).collect(),
        input_tokens: (known_tokens || unknown_tokens == 0 && !has_history_gap).then_some(input),
        output_tokens: (known_tokens || unknown_tokens == 0 && !has_history_gap).then_some(tokens_out),
        unknown_token_attempts: unknown_tokens,
        reported_usd: reported,
        estimated_usd: estimated,
        unknown_api_cost_attempts: unknown_costs,
        subscription_attempts: subscriptions,
        unassigned_native_runs: output.unassigned_native_runs,
        unknown_history_gaps: output.unknown_history_sources.len(),
        unknown_history_sources: output.unknown_history_sources,
    })
}
