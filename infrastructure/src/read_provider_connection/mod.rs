use application::error_context::ResultExt;
use std::path::Path;

use application::ports::PortFuture;

mod openrouter;
mod record;

pub fn read_provider_connection(
    directory: &Path,
) -> impl Fn(
    domain::ProviderId,
) -> PortFuture<Result<domain::ProviderConnection, Box<dyn std::error::Error + Send + Sync>>>
+ Send
+ Sync
+ use<> {
    let directory = directory.to_owned();
    move |provider_id| {
        let directory = directory.clone();
        Box::pin(async move {
            if provider_id.0 == "openrouter" {
                return openrouter::read(&directory.join("openrouter.json")).await;
            }
            if provider_id.0 != "openai" {
                return Err("Provider is not supported".into());
            }
            let path = directory.join("openai.json");
            tokio::fs::try_exists(&path)
                .await?
                .then_some(())
                .ok_or("No stored provider connection")
                .context("Sign in with /auth before starting a conversation")?;
            let credentials = record::load(&path).await?;
            let registration = credentials
                .active_client_id
                .as_ref()
                .and_then(|id| credentials.registrations.get(id))
                .ok_or("No active provider registration")
                .context("Sign in with /auth before starting a conversation")?;
            Ok(domain::ProviderConnection {
                id: registration.connection_id.clone(),
                provider_id,
            })
        })
    }
}
