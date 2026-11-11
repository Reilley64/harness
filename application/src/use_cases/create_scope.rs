mod dependencies;
mod error;
mod output;

pub use dependencies::Dependencies;
pub use error::Error;
pub use output::Output;

pub async fn call(
    dependencies: impl Into<Dependencies>,
    parent_id: String,
    execution_id: String,
    id: String,
    text: String,
    replaces: Option<String>,
    supersedes_run: Option<String>,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<Output, Error> {
    let d = dependencies.into();
    for value in [&parent_id, &execution_id, &id]
        .into_iter()
        .chain(replaces.iter())
        .chain(supersedes_run.iter())
    {
        if !uuid::Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == *value) {
            return Err(Error::caused_by("Scope identities must be canonical UUIDs"));
        }
    }
    let scope = domain::ScopeAmendment {
        id: id.clone(),
        execution_id,
        version: uuid::Uuid::new_v4().to_string(),
        text,
        replaces,
        supersedes_run,
        approved_by: None,
    };
    let response = d
        .update_coordination
        .call(
            parent_id,
            domain::CoordinationCommand::RequestScope(scope),
            None,
            cancellation,
        )
        .await
        .map_err(Error::caused_by)?;
    let scope = response
        .coordination
        .scopes
        .into_iter()
        .find(|scope| scope.id == id)
        .ok_or_else(|| Error::caused_by("Durable scope request is missing after acceptance"))?;
    Ok(Output { scope })
}
