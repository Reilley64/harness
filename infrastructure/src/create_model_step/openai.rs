use crate::tool_contracts;
use application::error_context::ResultExt;

use std::path::Path;
use std::time::Duration;

use application::ports::CreateModelStepError;
use application::ports::PresentTurnEvent;
use tokio_util::sync::CancellationToken;

use crate::create_provider_connection::settings::OpenaiSettings as AuthenticationSettings;

use super::ModelStepError;
use super::ModelStepFuture;
use super::OpenaiSettings;
use super::credentials;
use super::stream::Decoder;

pub(crate) fn create_model_step(
    settings: &OpenaiSettings,
    authentication: &AuthenticationSettings,
    credentials_path: &Path,
    read_model_allowlist: application::ports::ReadModelAllowlist,
) -> impl Fn(
    domain::Conversation,
    Vec<domain::Tool>,
    bool,
    PresentTurnEvent,
    CancellationToken,
) -> ModelStepFuture
+ Send
+ Sync
+ use<> {
    let settings = settings.clone();
    let authentication = authentication.clone();
    let credentials_path = credentials_path.to_owned();
    move |conversation, tools, finalizing, present, cancellation| {
        let settings = settings.clone();
        let authentication = authentication.clone();
        let credentials_path = credentials_path.clone();
        let read_model_allowlist = read_model_allowlist.clone();
        Box::pin(async move {
            let mut reported_usage = None;
            let result: Result<domain::ModelStep, ModelStepError> = async {
            if settings.timeout_seconds == 0 {
                return Err("Reply timeout must be greater than zero".into());
            }
            let url = super::checked_url(&format!(
                "{}/responses",
                settings.base_url.trim_end_matches('/')
            ))?;
            let token = credentials::access_token(
                &credentials_path,
                &conversation.provider_connection_id,
                &authentication,
                &cancellation,
            )
            .await.context("Authentication blocked: the saved connection could not authorize inference; sign in with /auth")?;
            if cancellation.is_cancelled() {
                return Err("Model generation was cancelled".into());
            }
            let stream = tokio::time::timeout(
                Duration::from_secs(settings.timeout_seconds),
                async {
                    let http = reqwest::Client::builder()
                        .redirect(reqwest::redirect::Policy::none())
                        .connect_timeout(Duration::from_secs(15))
                        .build()?;
                    let input = super::context::input(&conversation, finalizing)?;
                    let mut body = serde_json::json!({ "model": conversation.model.id, "input": input, "store": false, "stream": true, "include":["reasoning.encrypted_content"] });
                    if !tools.is_empty() {
                        body["tools"] = tool_contracts::definitions(&tools);
                        body["parallel_tool_calls"] = serde_json::json!(false);
                    }
                    if let Some(effort) = conversation.effort {
                        body["reasoning"] = serde_json::json!({ "effort": match effort {
                        domain::ModelEffort::Low => "low",
                        domain::ModelEffort::Medium => "medium",
                        domain::ModelEffort::High => "high",
                        domain::ModelEffort::ExtraHigh => "xhigh",
                    } });
                    }
                    let policy = read_model_allowlist.call().await
                        .context("Allowlist configuration could not be read; open /model settings")?;
                    if !policy.permits(&conversation.model.provider_id, &conversation.model.id) {
                        return Err::<domain::ModelStep, ModelStepError>("Model identity is outside the positive allowlist".into())
                            .context("Allowlist blocked: this provider/model is no longer permitted; open /model settings").map_err(Into::into);
                    }
                    let mut response = http
                        .post(url)
                        .bearer_auth(token)
                        .json(&body)
                        .send()
                        .await
                        .context("Could not contact the model provider")?;
                    if !response.status().is_success() {
                        let message = match response.status().as_u16() {
                            401 => "Model request failed (HTTP 401); sign in with /auth again",
                            403 => "Model request failed (HTTP 403); check account and model access",
                            429 => "Model request failed (HTTP 429); check usage limits and retry later",
                            _ => "The provider rejected the model request; check authentication, model access and usage limits",
                        };
                        return Err(format!("Model request failed (HTTP {})", response.status().as_u16()))
                            .context(message).map_err(Into::into);
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
                                reported_usage = super::usage::read(&event)?;
                            }
                            match kind {
                                "response.output_text.delta" | "response.refusal.delta" => {
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
                                        message_id: conversation
                                            .messages
                                            .last()
                                            .ok_or("Missing assistant message")?
                                            .id
                                            .clone(),
                                        delta: delta.to_owned(),
                                    })?;
                                }
                                "response.output_item.done" => {
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
                                            message_id: conversation
                                                .messages
                                                .last()
                                                .ok_or("Missing assistant message")?
                                                .id
                                                .clone(),
                                            delta: missing.into(),
                                        })?;
                                        text = completed_text;
                                    }
                                    return super::context::step(
                                        output,
                                        text,
                                        if refused {
                                            domain::MessageStatus::Refused
                                        } else {
                                            domain::MessageStatus::Completed
                                        },
                                        conversation
                                            .messages
                                            .last()
                                            .ok_or("Missing assistant message")?
                                            .content
                                            .len(),
                                        reported_usage,
                                    );
                                }
                                "response.incomplete" => {
                                    return Ok(domain::ModelStep {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        output: "[]".into(),
                                        calls: Vec::new(),
                                        status: domain::MessageStatus::Incomplete,
                                        usage: reported_usage,
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
                    Err("Missing provider completion")
                        .context("Provider stream ended without a terminal response")
                        .map_err(Into::into)
                },
            );
            tokio::select! {
                biased;
                result = stream => result.context("The model reply timed out")?,
                _ = cancellation.cancelled() => Err("Model generation was cancelled".into()),
            }
            }.await;
            result.map_err(|error| CreateModelStepError::from(error).with_usage(reported_usage))
        })
    }
}
