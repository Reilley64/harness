use application::error_context::ResultExt;

use std::time::Duration;

use application::ports::CreateModelStepError;
use application::ports::PresentTurnEvent;
use tokio_util::sync::CancellationToken;

use super::ModelStepError;
use super::ModelStepFuture;
use super::stream::Decoder;

pub(super) fn create_model_step<P>(
    base_url: &str,
    timeout_seconds: u64,
    prepare: P,
    read_model_allowlist: application::ports::ReadModelAllowlist,
    read_models: application::ports::ReadModels,
    read_work: application::ports::ReadWork,
    read_coordination: application::ports::ReadCoordination,
) -> impl Fn(
    domain::Conversation,
    Vec<domain::Tool>,
    bool,
    PresentTurnEvent,
    CancellationToken,
) -> ModelStepFuture
+ Send
+ Sync
+ use<P>
where
    P: Fn(
            domain::Conversation,
            Vec<domain::Tool>,
            bool,
            CancellationToken,
        )
            -> application::ports::PortFuture<Result<(String, serde_json::Value), ModelStepError>>
        + Send
        + Sync
        + 'static,
{
    let base_url = base_url.to_owned();
    let prepare = std::sync::Arc::new(prepare);
    move |conversation, tools, finalizing, present, cancellation| {
        let base_url = base_url.clone();
        let prepare = prepare.clone();
        let read_models = read_models.clone();
        let read_work = read_work.clone();
        let read_coordination = read_coordination.clone();
        let read_model_allowlist = read_model_allowlist.clone();
        Box::pin(async move {
            let mut reported_usage = None;
            let mut reported_cost = None;
            let mut attempted = false;
            let mut received_output = false;
            let mut provider_failure = None;
            let mut request_scope = None;
            let result: Result<domain::ModelStep, ModelStepError> = async {
            if timeout_seconds == 0 {
                return Err("Reply timeout must be greater than zero".into());
            }
            let url = super::checked_url(&format!(
                "{}/responses",
                base_url.trim_end_matches('/')
            ))?;

            let (model, effort, assignment, coordinator_id, research, connection_id, instructions, parent_run, reply_metadata, conversation) = tokio::task::spawn_blocking(move || {
                let reply_metadata = conversation.messages.last().map(|message| (message.id.clone(), message.content.len()));
                (
                    conversation.model.clone(),
                    conversation.effort,
                    conversation.assignment.clone(),
                    conversation.id.clone(),
                    conversation.research.clone(),
                    conversation.provider_connection_id.clone(),
                    conversation.selected_instructions.clone(),
                    conversation.current_work.as_ref().and_then(|_| conversation.runs.last().map(|run| run.id.clone())),
                    reply_metadata,
                    conversation,
                )
            }).await?;
            request_scope = Some(domain::ProviderFailure {
                provider_id: model.provider_id.clone(),
                connection_id: connection_id.clone(),
                model_id: model.id.clone(),
                kind: domain::ProviderFailureKind::Transient,
                retry_at: None,
            });
            let (token, body) = prepare(conversation, tools, finalizing, cancellation.clone()).await?;
            if cancellation.is_cancelled() {
                return Err("Model generation was cancelled".into());
            }

            let stream = tokio::time::timeout(
                Duration::from_secs(timeout_seconds),
                async {
                    let http = reqwest::Client::builder()
                        .redirect(reqwest::redirect::Policy::none())
                        .retry(reqwest::retry::never())
                        .connect_timeout(Duration::from_secs(15))
                        .build()?;
                    let body = tokio::task::spawn_blocking(move || serde_json::to_vec(&body)).await??;

                    let current = read_models.call().await.context("Capability blocked: the current model catalog is unavailable")?
                        .into_iter().find(|current| current.provider_id == model.provider_id && current.id == model.id)
                        .ok_or("The selected model is no longer in the catalog")
                        .context("Capability blocked: the selected model is unavailable; choose a supported model with /model")?;
                    if current.canonical_id != model.canonical_id {
                        return Err("The catalog canonical identity changed after request preparation")
                            .context("Capability blocked: the model identity changed during admission; retry with the current catalog").map_err(Into::into);
                    }
                    current.validate_effort(effort).context("Capability blocked: this effort is unsupported; choose an effort with /model")?;
                    current.validate_worker().context("Capability blocked: this identity is not a compatible worker; choose a supported model with /model")?;
                    let policy = read_model_allowlist.call().await
                        .context("Allowlist configuration could not be read; open /model settings")?;
                    if !policy.permits(&model.provider_id, &model.id) {
                        return Err::<domain::ModelStep, ModelStepError>("Model identity is outside the positive allowlist".into())
                            .context("Allowlist blocked: this provider/model is no longer permitted; open /model settings").map_err(Into::into);
                    }
                    if let Some(a)=&assignment {
                        let work=read_work.call(a.work_id.clone()).await?.ok_or("Approved work is unavailable")?;
                        work.authorize_assignment(a,&coordinator_id).context("Approved work changed; inspect and approve the current scope/criteria version")?;
                    }

                    application::research_access::read(&read_coordination, &read_work, research.as_ref(), &coordinator_id, &current, effort, &connection_id, instructions.as_deref()).await?;
                    if let Some(run_id) = &parent_run {
                        let state = read_coordination.call(coordinator_id.clone()).await?.ok_or("Required current-work coordination is unavailable")?;
                        if state.gate != domain::CoordinationGate::Open || state.run_id.as_ref() != Some(run_id) || state.controller.is_none() {
                            return Err("Parent coordination changed or is gated before HTTP admission".into());
                        }
                    }
                    if cancellation.is_cancelled() {
                        return Err("Model generation was cancelled before HTTP admission".into());
                    }
                    attempted = true;
                    let mut response = http
                        .post(url)
                        .bearer_auth(token)
                        .header(reqwest::header::CONTENT_TYPE, "application/json")
                        .body(body)
                        .send()
                        .await
                        .inspect_err(|error| {
                            if error.is_connect() || error.is_timeout() || error.is_request() {
                                provider_failure = request_scope.clone();
                            }
                        })
                        .context("Could not contact the model provider")?;
                    if !response.status().is_success() {
                        let status = response.status().as_u16();
                        let retry_at = super::failure::retry_at(
                            response.headers().get(reqwest::header::RETRY_AFTER).and_then(|value| value.to_str().ok()),
                            super::failure::now()?,
                        );
                        provider_failure = request_scope.clone().map(|mut failure| {
                            failure.kind = super::failure::kind(status, None);
                            failure.retry_at = retry_at;
                            failure
                        });
                        let body = super::failure::body(&mut response).await?;
                        provider_failure = request_scope.clone().map(|mut failure| {
                            failure.kind = super::failure::response_kind(&model.provider_id.0, status, &body);
                            failure.retry_at = retry_at;
                            failure
                        });
                        let accounting = serde_json::json!({"response": {"usage": body.get("usage")}});
                        let tokens = super::usage::read(&accounting);
                        let cost = super::usage::cost(&accounting);
                        if let Ok(tokens) = &tokens { reported_usage = *tokens; }
                        if let Ok(cost) = &cost { reported_cost = *cost; }
                        tokens?;
                        cost?;
                        return Err(format!("Model request failed (HTTP {status})"))
                            .context("The provider rejected the model request; inspect provider wait or authentication and model settings").map_err(Into::into);
                    }

                    if let Some(scope) = &mut request_scope {
                        scope.retry_at = super::failure::retry_at(response.headers().get(reqwest::header::RETRY_AFTER).and_then(|value| value.to_str().ok()), super::failure::now()?);
                    }
                    let media_type = response
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.split(';').next())
                        .map(str::trim)
                        .filter(|value| !value.is_empty());
                    if let Some(value) = media_type
                        && !value.eq_ignore_ascii_case("text/event-stream")
                    {
                        let response_type = [
                            "application/json",
                            "application/problem+json",
                            "text/html",
                            "text/plain",
                            "application/octet-stream",
                        ]
                        .into_iter()
                        .find(|known| value.eq_ignore_ascii_case(known))
                        .unwrap_or("other content type");
                        return Err(format!(
                            "HTTP {} {response_type}: expected text/event-stream",
                            response.status().as_u16()
                        )
                        .into());
                    }
                    let headerless = media_type.is_none();
                    let mut received_content = false;
                    let mut received_event = false;
                    let mut decoder = Decoder::default();
                    let mut refused = false;
                    let mut text = String::new();
                    let mut items = std::collections::BTreeMap::new();
                    while let Some(bytes) = response
                        .chunk()
                        .await
                        .inspect_err(|_| {
                            if !received_output { provider_failure = request_scope.clone(); }
                        })
                        .context("Provider stream was interrupted")?
                    {
                        if !received_content
                            && let Some(first) =
                                bytes.iter().find(|byte| !byte.is_ascii_whitespace())
                        {
                            received_content = true;
                            if headerless {
                                match first {
                                    b'{' | b'[' => {
                                        return Err(format!(
                                            "HTTP {}: JSON-like body instead of an SSE stream",
                                            response.status().as_u16()
                                        )
                                        .into());
                                    }
                                    b'<' => {
                                        return Err(format!(
                                            "HTTP {}: HTML/XML-like body instead of an SSE stream",
                                            response.status().as_u16()
                                        )
                                        .into());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        for event in decoder.push(&bytes)? {
                            received_event = true;
                            let kind = event
                                .get("type")
                                .and_then(serde_json::Value::as_str)
                                .ok_or("Provider event has no type")?;
                            if matches!(kind, "response.completed" | "response.incomplete" | "response.failed" | "error") {
                                if matches!(kind, "response.failed" | "error") && !received_output {
                                    provider_failure = request_scope.clone().map(|mut failure| {
                                        failure.kind = super::failure::event_kind(&event);
                                        failure
                                    });
                                }
                                let tokens = super::usage::read(&event);
                                let cost = super::usage::cost(&event);
                                if let Ok(tokens) = &tokens { reported_usage = *tokens; }
                                if let Ok(cost) = &cost { reported_cost = *cost; }
                                tokens?;
                                cost?;
                            }
                            match kind {
                                "response.output_text.delta" | "response.refusal.delta" => {
                                    received_output = true;
                                    refused |= kind == "response.refusal.delta";
                                    let delta = event
                                        .get("delta")
                                        .and_then(serde_json::Value::as_str)
                                        .ok_or("Invalid provider text delta")?;
                                    if text.len() + delta.len() > 1_048_576 {
                                        return Err("Model text exceeds 1 MiB".into());
                                    }
                                    text.push_str(delta);
                                    present.call(domain::TurnEvent::Text {
                                        message_id: reply_metadata.as_ref().ok_or("Missing assistant message")?.0.clone(),
                                        delta: delta.to_owned(),
                                    })?;
                                }
                                "response.output_item.done" => {
                                    received_output = true;
                                    if let (Some(index), Some(item)) = (
                                        event
                                            .get("output_index")
                                            .and_then(serde_json::Value::as_u64),
                                        event.get("item"),
                                    ) {
                                        if index > 1024
                                            || serde_json::to_vec(item)?.len() > 1_048_576
                                        {
                                            return Err(
                                                "Provider output item exceeds its bound".into()
                                            );
                                        }
                                        items.insert(index, item.clone());
                                        if serde_json::to_vec(&items)?.len() > 1_048_576 {
                                            return Err("Provider output exceeds 1 MiB".into());
                                        }
                                    }
                                }
                                "response.completed" => {
                                    let resolved_model_id = event.pointer("/response/model").and_then(serde_json::Value::as_str).filter(|s| !s.is_empty() && s.len() <= 512 && !s.chars().any(char::is_control)).map(str::to_owned);
                                    if event.pointer("/response/model").is_some_and(|value| !value.is_null()) && resolved_model_id.is_none() {
                                        return Err("The provider returned an invalid model identity; no tools were executed".into());
                                    }
                                    let serving_provider = event.pointer("/response/provider").and_then(serde_json::Value::as_str).filter(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)).map(str::to_owned);
                                    if current.provider_id.0 == "openrouter" && resolved_model_id.as_ref().is_some_and(|id| id != &current.id && Some(id) != current.canonical_id.as_ref()) {
                                        return Err("OpenRouter returned an unexpected model identity; no tools were executed").context("The provider returned a different model; no fallback is permitted").map_err(Into::into);
                                    }
                                    let output = event
                                        .pointer("/response/output")
                                        .and_then(serde_json::Value::as_array)
                                        .filter(|output| !output.is_empty())
                                        .cloned()
                                        .unwrap_or_else(|| items.values().cloned().collect());
                                    let completed_text = output
                                        .iter()
                                        .filter(|item| item["type"] == "message")
                                        .filter_map(|item| item["content"].as_array())
                                        .flatten()
                                        .filter_map(|part| {
                                            part["text"]
                                                .as_str()
                                                .or_else(|| part["refusal"].as_str())
                                        })
                                        .collect::<String>();
                                    if !completed_text.is_empty() && completed_text != text {
                                        if !completed_text.starts_with(&text) {
                                            return Err(
                                                "Provider terminal text differs from its stream"
                                                    .into(),
                                            );
                                        }
                                        let missing = &completed_text[text.len()..];
                                        if completed_text.len() > 1_048_576 {
                                            return Err("Model text exceeds 1 MiB".into());
                                        }
                                        present.call(domain::TurnEvent::Text {
                                            message_id: reply_metadata.as_ref().ok_or("Missing assistant message")?.0.clone(),
                                            delta: missing.into(),
                                        })?;
                                        text = completed_text;
                                    }
                                    let mut step = super::context::step(
                                        output,
                                        text,
                                        if refused {
                                            domain::MessageStatus::Refused
                                        } else {
                                            domain::MessageStatus::Completed
                                        },
                                        reply_metadata.as_ref().ok_or("Missing assistant message")?.1,
                                        reported_usage,
                                    )?;
                                    step.reported_cost = reported_cost;
                                    step.resolved_model_id = resolved_model_id;
                                    step.catalog_canonical_id = current.canonical_id.clone();
                                    step.serving_provider = serving_provider;
                                    return Ok(step);
                                }
                                "response.incomplete" => {
                                    return Ok(domain::ModelStep {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        output: "[]".into(),
                                        resolved_model_id: None,
                                        catalog_canonical_id: current.canonical_id.clone(),
                                        serving_provider: None,
                                        calls: Vec::new(),
                                        status: domain::MessageStatus::Incomplete,
                                        usage: reported_usage,
                                        reported_cost,
                                    });
                                }
                                "error" | "response.failed" => {
                                    let code = event
                                        .pointer("/response/error/code")
                                        .or_else(|| event.pointer("/error/code"))
                                        .or_else(|| event.get("code"))
                                        .and_then(serde_json::Value::as_str)
                                        .filter(|code| {
                                            code.len() < 100
                                                && code.chars().all(|character| {
                                                    character.is_ascii_alphanumeric()
                                                        || character == '_'
                                                })
                                        })
                                        .unwrap_or("generation_failed");
                                    if !received_output {
                                        provider_failure = request_scope.clone().map(|mut failure| {
                                            failure.kind = super::failure::event_kind(&event);
                                            failure
                                        });
                                    }
                                    return Err(format!("Provider reply failed: {code}").into());
                                }
                                _ => {}
                            }
                        }
                    }
                    if !received_content {
                        return Err(format!(
                            "HTTP {}: empty response; expected text/event-stream",
                            response.status().as_u16()
                        )
                        .into());
                    }
                    if headerless && !received_event {
                        return Err(format!(
                            "HTTP {}: non-SSE response; expected Responses events",
                            response.status().as_u16()
                        )
                        .into());
                    }
                    if !received_output { provider_failure = request_scope.clone(); }
                    Err("Missing provider completion")
                        .context("Provider stream ended without a terminal response")
                        .map_err(Into::into)
                },
            );
            tokio::select! {
                biased;
                result = stream => match result {
                    Ok(result) => result,
                    Err(error) => {
                        if attempted && !received_output { provider_failure = request_scope.clone(); }
                        Err(error).context("The model reply timed out").map_err(Into::into)
                    }
                },
                _ = cancellation.cancelled() => Err("Model generation was cancelled".into()),
            }
            }.await;
            result.map_err(|error| {
                let mut error = CreateModelStepError::from(error)
                    .with_usage(reported_usage)
                    .with_provider_failure(provider_failure);
                error.reported_cost = reported_cost;
                if attempted {
                    error
                } else {
                    error.without_attempt()
                }
            })
        })
    }
}
