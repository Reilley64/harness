mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    id: String,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    let decision = d
        .read_decision
        .call(id)
        .await
        .map_err(Error::caused_by)?
        .ok_or_else(|| Error::caused_by("Assignment intent was not found"))?;
    let candidates = decision.request.map(|r| r.candidates).unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(Error::caused_by)?
        .as_secs();
    let response = d
        .read_routing_evidence
        .call(candidates, now, true, cancellation)
        .await
        .map_err(Error::caused_by)?;
    Ok(Output {
        evidence: response.evidence,
        diagnostics: response.diagnostics,
        checked_at: now,
    })
}
