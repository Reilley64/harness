use crate::error_context::ErrorContext;
use crate::error_context::ResultExt;
use crate::model_access;
use crate::ports;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) async fn native(
    models: &ports::ReadModels,
    policy: &ports::ReadModelAllowlist,
    connections: &ports::ReadProviderConnection,
    id: &str,
) -> Result<(domain::Model, domain::ProviderConnection), Error> {
    let model = models
        .call()
        .await
        .context("Capability blocked: the Harness catalog could not be read")?
        .into_iter()
        .find(|m| m.provider_id.0 == "openrouter" && m.id == id)
        .ok_or("Native identity is absent")
        .context("Capability blocked: native Decider is missing from the Harness catalog")?;
    model.validate_decider().context(
        "Capability blocked: select a compatible native Decisions release with /decider",
    )?;
    let policy = policy
        .call()
        .await
        .context("Allowlist blocked: the positive model policy could not be read")?;
    if !policy.permits(&model.provider_id, &model.id) {
        return Err(ErrorContext::caused_by(
            "Allowlist blocked: configure the selected native Decider in /model settings",
            "Selected native identity is not permitted",
        )
        .into());
    }
    let connection = connections
        .call(model.provider_id.clone())
        .await
        .context("Authentication blocked: connect the native OpenRouter provider with /auth")?;
    Ok((model, connection))
}

pub(crate) async fn candidate(
    models: &ports::ReadModels,
    policy: &ports::ReadModelAllowlist,
    connections: &ports::ReadProviderConnection,
    factory: &ports::CreateModelStepFactory,
    selection: &domain::ModelSelection,
    context_tokens: u64,
) -> Result<domain::DecisionCandidate, Error> {
    let model = model_access::read_model(
        models,
        &selection.provider_id,
        &selection.model_id,
        selection.effort,
    )
    .await?;
    let connection = model_access::read(policy, connections, factory, &model).await?;
    let candidate = domain::DecisionCandidate {
        key: uuid::Uuid::new_v4().to_string(),
        selection: selection.clone(),
        connection_id: connection.id,
        model,
    };
    candidate.validate(context_tokens)?;
    Ok(candidate)
}
