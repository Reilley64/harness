use std::path::Path;

use application::ports::PortFuture;
use application::ports::PresentAuthorization;

use crate::create_provider_connection::AuthenticationError;
use crate::create_provider_connection::Settings;
use crate::create_provider_connection::openai;
use crate::create_provider_connection::openrouter;

pub fn create_provider_connection_factory(
    settings: &Settings,
    credentials_path: &Path,
    provider_id: &domain::ProviderId,
) -> Result<
    impl Fn(PresentAuthorization) -> PortFuture<Result<domain::ProviderConnection, AuthenticationError>>
    + Send
    + Sync
    + use<>,
    AuthenticationError,
> {
    match provider_id.0.as_str() {
        "openai" => {
            let create = openai::create_provider_connection(
                &settings.openai,
                &credentials_path.join("openai.json"),
            );
            Ok(Box::new(create)
                as Box<
                    dyn Fn(
                            PresentAuthorization,
                        )
                            -> PortFuture<Result<domain::ProviderConnection, AuthenticationError>>
                        + Send
                        + Sync,
                >)
        }
        "openrouter" => {
            let create = openrouter::create_provider_connection(
                &settings.openrouter,
                &credentials_path.join("openrouter.json"),
            );
            Ok(Box::new(create))
        }
        _ => Err(std::io::Error::other(format!(
            "Authentication is not supported for provider {provider_id}"
        ))
        .into()),
    }
}
