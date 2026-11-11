use crate::create_decision_inputs;
use crate::create_model_step::checked_url;
use crate::http;
use crate::storage;
use application::ports;
use std::collections::BTreeMap;
use std::path::Path;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub url: String,
}

#[derive(serde::Deserialize)]
struct Record {
    version: u32,
    connection_id: String,
    issuer: String,
    key: String,
}

#[derive(serde::Deserialize)]
struct Response {
    model: String,
    provider: Option<String>,
    answers: Option<serde_json::Value>,
    usage: UsageResponse,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum AnswerResponse {
    Choice {
        choice: String,
        confidence: Option<f64>,
        probabilities: Option<BTreeMap<String, f64>>,
    },
    Refusal,
}

#[derive(serde::Deserialize)]
struct UsageResponse {
    input_tokens: u64,
    output_tokens: u64,
    cost: Option<f64>,
}

pub fn create_native_decision(
    settings: &Settings,
    issuer: &str,
    credentials_path: &Path,
    read_models: ports::ReadModels,
    read_model_allowlist: ports::ReadModelAllowlist,
    read_work: ports::ReadWork,
    read_conversation: ports::ReadConversation,
) -> impl Fn(
    domain::AssignmentDecision,
    CancellationToken,
) -> ports::PortFuture<Result<domain::DecisionResponse, ports::CreateNativeDecisionError>>
+ Send
+ Sync
+ use<> {
    let url = settings.url.clone();
    let issuer = issuer.to_owned();
    let path = credentials_path.to_owned();
    move |decision, cancellation| {
        let url = url.clone();
        let issuer = issuer.clone();
        let path = path.clone();
        let read_models = read_models.clone();
        let read_model_allowlist = read_model_allowlist.clone();
        let read_work = read_work.clone();
        let read_conversation = read_conversation.clone();
        Box::pin(async move {
            let operation = async {
                let (decision, body, question) = tokio::task::spawn_blocking(move || {
                    if decision.legacy || decision.phase != domain::DecisionPhase::Pending {
                        return Err::<_, Error>("Only a claimed complete-pool pending stage can issue native inference; legacy replay is forbidden".into());
                    }
                    decision.validate()?;
                    let request = decision.request.as_ref().ok_or("Native request snapshot is missing")?;
                    let stage = decision.stages.iter().find(|s| s.started_at.is_some() && s.finished_at.is_none() && s.failure.is_none()).ok_or("No pending native stage")?;
                    stage.validate(request)?;
                    if create_decision_inputs::encode(&decision, &stage.members, &stage.id)? != stage.input {
                        return Err::<_, Error>("Persisted native input differs from its immutable snapshot".into());
                    }
                    let (encoded, question) = (stage.input.as_bytes().to_vec(), stage.id.clone());
                    if encoded.len() > 24576
                        || encoded.len() as u64 > request.model.context_window.ok_or("Native context unknown")?
                    {
                        return Err::<_, Error>(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported native input: complete bounded brief/pool exceeds conservative byte envelope; no truncation or inference").into());
                    }
                    Ok((decision, encoded, question))
                }).await??;
                let request = decision
                    .request
                    .as_ref()
                    .ok_or("Native request snapshot is missing")?;
                let destination = checked_url(&url)?;
                let authority = checked_url(&issuer)?;
                let bytes = storage::read_private(&path, 1_048_576).await?;
                let credentials: Record =
                    tokio::task::spawn_blocking(move || serde_json::from_slice(&bytes)).await??;
                if authority.path() != "/"
                    || credentials.version != 1
                    || credentials.connection_id != request.connection_id
                    || credentials.key.is_empty()
                    || credentials.key.len() > 16384
                    || credentials.key.chars().any(char::is_control)
                    || credentials.issuer != authority.origin().ascii_serialization()
                    || destination.origin() != authority.origin()
                {
                    return Err::<_, Error>(
                        "Authentication blocked: native connection/origin mismatch; use /auth"
                            .into(),
                    );
                }

                let current = read_models
                    .call()
                    .await?
                    .into_iter()
                    .find(|m| {
                        m.provider_id == request.model.provider_id && m.id == request.model.id
                    })
                    .ok_or("Native identity is no longer in the Harness catalog")?;
                current.validate_decider()?;
                if current
                    .context_window
                    .is_none_or(|limit| body.len() as u64 > limit)
                    || current.canonical_id != request.model.canonical_id
                    || !read_model_allowlist
                        .call()
                        .await?
                        .permits(&current.provider_id, &current.id)
                {
                    return Err("Native configuration changed before HTTP admission".into());
                }
                let parent = read_conversation.call(decision.parent_id.clone()).await?;
                let assignment = parent
                    .assignment
                    .as_ref()
                    .ok_or("CoS assignment authority is missing")?;
                let work = read_work
                    .call(decision.work_id.clone())
                    .await?
                    .ok_or("Approved work is missing")?;
                if parent.resume_required
                    || assignment.work_id != decision.work_id
                    || assignment.intent != decision.work_intent
                    || assignment.proposal.version != decision.work_version
                    || parent.agent.as_ref().is_none_or(|a| {
                        a.id.0 != decision.parent_agent_id
                            || a.role != domain::AgentRole::ChiefOfStaff
                    })
                {
                    return Err("Actual CoS authority or Resume gate changed".into());
                }
                work.authorize_assignment(assignment, &decision.parent_id)?;

                let client = reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .retry(reqwest::retry::never())
                    .build()?;
                let response = client
                    .post(destination)
                    .bearer_auth(credentials.key)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body)
                    .send()
                    .await?;
                if !response.status().is_success() {
                    return Err(format!(
                        "Native decision HTTP status {}; no SDK retry",
                        response.status().as_u16()
                    )
                    .into());
                }
                Ok((http::body(response, 1_048_576).await?, question))
            };
            tokio::pin!(operation);
            let (bytes, question) = tokio::select! {
                biased;
                _ = cancellation.cancelled() => {
                    Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Native decision stopped").into())
                },
                result = &mut operation => result,
            }.map_err(ports::CreateNativeDecisionError::from)?;

            tokio::task::spawn_blocking(move || decode(bytes, &question))
                .await
                .map_err(|error| ports::CreateNativeDecisionError::from(Box::new(error) as Error))?
        })
    }
}

fn decode(
    bytes: Vec<u8>,
    question: &str,
) -> Result<domain::DecisionResponse, ports::CreateNativeDecisionError> {
    let response: Response = serde_json::from_slice(&bytes)
        .map_err(|error| ports::CreateNativeDecisionError::from(Box::new(error) as Error))?;
    let mut report = domain::DecisionResponse {
        choice: None,
        confidence: None,
        probabilities: Vec::new(),
        resolved_model: response.model,
        serving_provider: response.provider,
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
        cost: response.usage.cost,
    };
    report
        .validate_report()
        .map_err(|error| ports::CreateNativeDecisionError::from(Error::from(error)))?;

    let answer = (|| {
        let answers: BTreeMap<String, AnswerResponse> = serde_json::from_value(
            response
                .answers
                .ok_or("Native selection answer is missing")?,
        )?;
        if answers.len() != 1 {
            return Err::<_, Error>(
                "Native response must contain exactly the requested answer".into(),
            );
        }
        answers
            .into_iter()
            .find_map(|(key, answer)| (key == question).then_some(answer))
            .ok_or_else(|| Error::from("Native selection answer is missing"))
    })()
    .map_err(|error| ports::CreateNativeDecisionError::from(error).with_report(report.clone()))?;
    if let AnswerResponse::Choice {
        choice,
        confidence,
        probabilities,
    } = answer
    {
        report.choice = Some(choice);
        report.confidence = confidence;
        report.probabilities = probabilities.unwrap_or_default().into_iter().collect();
    }
    Ok(report)
}
