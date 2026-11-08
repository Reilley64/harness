mod record;

use self::record::Record;
use crate::cache::encode_bounded;
use crate::conversations;
use crate::storage;
use application::ports::PortFuture;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    intent: String,
    role: String,
    task: String,
    constraints: String,
    tool_context: String,
    context_tokens: u64,
}

pub(crate) fn decode(value: &str) -> Result<domain::ToolInvocation, Error> {
    let a: Arguments = serde_json::from_str(value)?;
    let brief = domain::AssignmentBrief {
        intent: a.intent,
        role: a.role,
        task: a.task,
        constraints: a.constraints,
        tool_context: a.tool_context,
        context_tokens: a.context_tokens,
    };
    brief.validate()?;
    Ok(domain::ToolInvocation::PrepareAssignment(brief))
}

pub(crate) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"prepare_assignment","description":"For the actual approved Chief of Staff only: persist one new child assignment intent and bounded untrusted brief, capturing the actual-parent model AND thinking. Same canonical UUID/exact payload reuses the intent; changed payload conflicts. This prepares selection/inspection only, never launches a worker or workspace. Inspect /decision <intent>. Do not put secrets or full transcripts in the brief.","parameters":{"type":"object","properties":{"intent":{"type":"string"},"role":{"type":"string","enum":["research","implementer","reviewer"]},"task":{"type":"string","maxLength":4096},"constraints":{"type":"string","maxLength":2048},"toolContext":{"type":"string","maxLength":2048},"contextTokens":{"type":"integer","minimum":1,"maximum":1000000000}},"required":["intent","role","task","constraints","toolContext","contextTokens"],"additionalProperties":false}})
}

pub fn update_decision(
    directory: &Path,
) -> impl Fn(
    domain::AssignmentDecision,
    Option<u64>,
) -> PortFuture<Result<domain::AssignmentDecision, Error>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |mut decision, revision| {
        let directory = directory.clone();
        Box::pin(async move {
            conversations::validate_id(&decision.id)?;
            let path = directory.join(format!("{}.json", decision.id));
            let _guard = loop {
                match storage::lock(&path).await {
                    Ok(guard) => break guard,
                    Err(error)
                        if error
                            .source()
                            .and_then(|e| e.downcast_ref::<std::fs::TryLockError>())
                            .is_some_and(|e| matches!(e, std::fs::TryLockError::WouldBlock)) =>
                    {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                    Err(error) => return Err(error),
                }
            };
            let bytes = match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => Some(storage::read_private(&path, 2 * 1024 * 1024).await?),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.into()),
            };
            let (decision, encoded) = tokio::task::spawn_blocking(move || {
                let prior: Option<domain::AssignmentDecision> = bytes.map(|b| serde_json::from_slice::<Record>(&b)).transpose()?.map(TryInto::try_into).transpose()?;
                if let Some(prior) = &prior {
                    if !prior.same_intent(&decision) { return Err::<_, Error>("Assignment intent conflicts with an exact accepted payload".into()); }
                    if revision.is_none() { return Ok((prior.clone(), None)); }
                    if Some(prior.revision) != revision { return Err("Stale decision revision; inspect existing intent without repeating inference".into()); }
                    prior.validate_transition(&decision)?;
                } else if revision.is_some() || decision.revision != 0 || decision.phase != domain::DecisionPhase::Prepared {
                    return Err("Invalid initial assignment claim".into());
                }
                decision.validate()?;
                decision.revision = decision.revision.checked_add(1).ok_or("Decision revision exhausted")?;
                let encoded = encode_bounded(&Record::from(decision.clone()), 2 * 1024 * 1024)?;
                let _: domain::AssignmentDecision = serde_json::from_slice::<Record>(&encoded)?.try_into()?;
                Ok((decision, Some(encoded)))
            }).await??;
            if let Some(encoded) = encoded {
                storage::replace(&path, &encoded).await?;
            }
            Ok(decision)
        })
    }
}
