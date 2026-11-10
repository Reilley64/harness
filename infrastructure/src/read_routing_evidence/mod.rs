use crate::cache::encode_bounded;
use crate::create_model_step::checked_url;
use crate::http;
use crate::storage;
use application::ports::PortFuture;
use sha2::Digest;
use std::path::Path;
use tokio_util::sync::CancellationToken;

type Error = Box<dyn std::error::Error + Send + Sync>;
const LIMIT: usize = 16 * 1024 * 1024;

pub struct Response {
    pub evidence: Vec<(String, Vec<domain::RoutingEvidence>)>,
    pub diagnostics: Vec<Error>,
}

#[ohno::error]
#[display("Routing evidence cache commit failed")]
pub struct CacheCommitError {
    pub source_errors: Vec<Error>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: u32,
    url: String,
    last_refresh: String,
    failed_at: Option<u64>,

    retrieved_at: Option<u64>,
    revision: Option<String>,
    rows: Option<Vec<RowResponse>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct RowResponse {
    id: String,
    provider_id: String,
    canonical_id: Option<String>,
    #[serde(default)]
    routing_evidence: Vec<EvidenceResponse>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceResponse {
    category: String,
    metric: String,
    value: Option<f64>,
    unit: String,

    source: String,
    source_url: String,
    revision: String,
    measured_at: Option<u64>,
    published_at: Option<u64>,
    effective_at: Option<u64>,

    requested_model: String,
    resolved_model: Option<String>,
    serving_provider: Option<String>,
    effort: Option<String>,
    configuration: String,
}

fn effort(value: Option<&str>) -> Result<Option<domain::ModelEffort>, Error> {
    value
        .map(|s| {
            Ok(match s {
                "low" => domain::ModelEffort::Low,
                "medium" => domain::ModelEffort::Medium,
                "high" => domain::ModelEffort::High,
                "extra_high" => domain::ModelEffort::ExtraHigh,
                _ => return Err::<_, Error>("Unsupported evidence effort".into()),
            })
        })
        .transpose()
}

fn fact(
    e: &EvidenceResponse,
    retrieved_at: Option<u64>,
    refresh_failed: bool,
) -> Result<domain::RoutingEvidence, Error> {
    let fact = domain::RoutingEvidence {
        category: e.category.clone(),
        metric: e.metric.clone(),
        value: e.value,
        unit: e.unit.clone(),
        source: e.source.clone(),
        source_url: e.source_url.clone(),
        revision: Some(e.revision.clone()),
        retrieved_at,
        measured_at: e.measured_at,
        published_at: e.published_at,
        effective_at: e.effective_at,
        refresh_failed,
        requested_model: e.requested_model.clone(),
        resolved_model: e.resolved_model.clone(),
        serving_provider: e.serving_provider.clone(),
        effort: effort(e.effort.as_deref())?,
        configuration: e.configuration.clone(),
    };
    fact.validate()?;
    checked_url(&fact.source_url)?;
    if fact.category == "benchmark"
        && fact.value.is_some()
        && (fact.measured_at.is_none() || fact.configuration.is_empty() || fact.unit.is_empty())
    {
        return Err(
            "Benchmark observations require original measurement, configuration and units".into(),
        );
    }
    Ok(fact)
}

fn validate(record: &Record) -> Result<(), Error> {
    let complete =
        record.retrieved_at.is_some() && record.revision.is_some() && record.rows.is_some();
    let absent =
        record.retrieved_at.is_none() && record.revision.is_none() && record.rows.is_none();
    if record.format != 1
        || !(complete || absent)
        || record.last_refresh == "available" && (!complete || record.failed_at.is_some())
        || record.last_refresh == "failed" && record.failed_at.is_none()
        || !matches!(record.last_refresh.as_str(), "available" | "failed")
        || record
            .revision
            .as_ref()
            .is_some_and(|r| r.len() != 64 || !r.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("Unsupported or inconsistent routing evidence cache lifecycle".into());
    }
    let mut seen = std::collections::HashSet::new();
    for row in record.rows.iter().flatten() {
        if !seen.insert((&row.provider_id, &row.id)) || row.routing_evidence.len() > 32 {
            return Err("Duplicate or oversized routing evidence".into());
        }
        for e in &row.routing_evidence {
            fact(e, record.retrieved_at, false)?;
        }
    }
    Ok(())
}

async fn persist(path: &Path, record: Record) -> Result<Record, Error> {
    let (record, bytes) = tokio::task::spawn_blocking(move || {
        validate(&record)?;
        let bytes = encode_bounded(&record, LIMIT)?;
        Ok::<_, Error>((record, bytes))
    })
    .await??;
    storage::private_directory(path.parent().ok_or("Evidence cache has no directory")?).await?;
    storage::replace(path, &bytes).await?;
    Ok(record)
}

pub fn read_routing_evidence(
    url: &str,
    path: &Path,
) -> impl Fn(
    Vec<domain::DecisionCandidate>,
    u64,
    bool,
    CancellationToken,
) -> PortFuture<Result<Response, Error>>
+ Send
+ Sync
+ use<> {
    let url = url.to_owned();
    let path = path.to_owned();
    move |candidates, now, refresh, cancellation| {
        let url = url.clone();
        let path = path.clone();
        Box::pin(async move {
            let mut cached = match tokio::fs::symlink_metadata(&path).await {
                Ok(_) => {
                    let bytes = storage::read_private(&path, LIMIT).await?;
                    Some(
                        tokio::task::spawn_blocking(move || {
                            let record: Record = serde_json::from_slice(&bytes)?;
                            validate(&record)?;
                            Ok::<_, Error>(record)
                        })
                        .await??,
                    )
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.into()),
            };
            if cached.as_ref().is_some_and(|r| r.url != url) {
                cached = None;
            }
            let due = refresh
                || cached
                    .as_ref()
                    .and_then(|r| r.retrieved_at)
                    .is_none_or(|at| at > now || now.saturating_sub(at) >= 86_400);
            let mut diagnostics = Vec::new();
            if due {
                let fetched = async {
                    let destination = checked_url(&url)?;
                    let client = reqwest::Client::builder()
                        .redirect(reqwest::redirect::Policy::none())
                        .retry(reqwest::retry::never())
                        .timeout(std::time::Duration::from_secs(10))
                        .build()?;
                    let response = client.get(destination).send().await?.error_for_status()?;
                    let bytes = http::body(response, LIMIT).await?;
                    let source_url = url.clone();
                    tokio::task::spawn_blocking(move || {
                        let revision = format!("{:x}", sha2::Sha256::digest(&bytes));
                        let rows = serde_json::from_slice::<Vec<RowResponse>>(&bytes)?;
                        let record = Record {
                            format: 1,
                            url: source_url,
                            last_refresh: "available".into(),
                            failed_at: None,
                            retrieved_at: Some(now),
                            revision: Some(revision),
                            rows: Some(rows),
                        };
                        validate(&record)?;
                        Ok::<_, Error>(record)
                    })
                    .await?
                };
                let fetched = tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err("Routing evidence refresh stopped".into()),
                    result = fetched => result,
                };
                let record = match fetched {
                    Ok(record) => record,
                    Err(error) => {
                        diagnostics.push(error);
                        let mut record = cached.take().unwrap_or_else(|| Record {
                            format: 1,
                            url: url.clone(),
                            last_refresh: "failed".into(),
                            failed_at: Some(now),
                            retrieved_at: None,
                            revision: None,
                            rows: None,
                        });
                        record.last_refresh = "failed".into();
                        record.failed_at = Some(now);
                        record
                    }
                };
                cached = Some(persist(&path, record).await.map_err(|error| {
                    CacheCommitError::caused_by(std::mem::take(&mut diagnostics), error)
                })?);
            }
            let failed = cached.as_ref().is_some_and(|r| r.last_refresh == "failed");
            let evidence = tokio::task::spawn_blocking(move || {
                candidates.into_iter().map(|candidate| {
                    let row = cached.as_ref().and_then(|r| r.rows.as_ref()).and_then(|rows| rows.iter().find(|row| row.provider_id == candidate.model.provider_id.0 && row.id == candidate.model.id && row.canonical_id == candidate.model.canonical_id));
                    let mut facts = Vec::new();
                    if let Some(row) = row {
                        for e in &row.routing_evidence {
                            let e = fact(e, cached.as_ref().and_then(|r| r.retrieved_at), failed)?;
                            if e.requested_model == candidate.model.id && e.effort == candidate.selection.effort
                                && e.resolved_model.as_ref().is_none_or(|resolved| resolved == &candidate.model.id || Some(resolved) == candidate.model.canonical_id.as_ref())
                            {
                                facts.push(e);
                            }
                        }
                    }
                    for category in ["catalog", "capability", "pricing", "benchmark"] {
                        if !facts.iter().any(|f| f.category == category) {
                            facts.push(domain::RoutingEvidence {
                                category: category.into(), metric: "unavailable dated observation".into(), value: None, unit: if category == "pricing" { "USD per million tokens" } else { "unknown" }.into(),
                                source: "Harness-owned catalog".into(), source_url: url.clone(), revision: cached.as_ref().and_then(|r| r.revision.clone()),
                                retrieved_at: cached.as_ref().and_then(|r| r.retrieved_at), measured_at: None, published_at: None, effective_at: None, refresh_failed: failed,
                                requested_model: candidate.model.id.clone(), resolved_model: candidate.model.canonical_id.clone(), serving_provider: None, effort: candidate.selection.effort,
                                configuration: "No compatible dated source observation; catalog list prices are not subscription incremental cost".into(),
                            });
                        }
                    }
                    Ok::<_, Error>((candidate.key, facts))
                }).collect::<Result<Vec<_>, Error>>()
            }).await??;
            Ok(Response {
                evidence,
                diagnostics,
            })
        })
    }
}
