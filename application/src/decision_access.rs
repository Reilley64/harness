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
        .await?
        .into_iter()
        .find(|m| m.provider_id.0 == "openrouter" && m.id == id)
        .ok_or("Capability blocked: native Decider is missing from the Harness catalog")?;
    model.validate_decider()?;
    if !policy.call().await?.permits(&model.provider_id, &model.id) {
        return Err(
            "Allowlist blocked: configure the selected native Decider in /model settings".into(),
        );
    }
    let connection = connections.call(model.provider_id.clone()).await?;
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
