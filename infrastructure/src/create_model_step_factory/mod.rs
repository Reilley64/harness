use std::path::Path;

use application::ports::CreateModelStep;

use crate::create_model_step::ModelStepError;
use crate::create_model_step::Settings;
use crate::create_model_step::openai;
use crate::create_model_step::openrouter;
use crate::create_provider_connection::Settings as AuthenticationSettings;

pub fn create_model_step_factory(
    settings: &Settings,
    authentication: &AuthenticationSettings,
    credentials_path: &Path,
    provider_id: &domain::ProviderId,
    read_model_allowlist: application::ports::ReadModelAllowlist,
    read_models: application::ports::ReadModels,
) -> Result<CreateModelStep, ModelStepError> {
    match provider_id.0.as_str() {
        "openai" => {
            let create = openai::create_model_step(
                &settings.openai,
                &authentication.openai,
                &credentials_path.join("openai.json"),
                read_model_allowlist,
                read_models,
            );
            Ok(CreateModelStep::new(create))
        }
        "openrouter" => {
            let create = openrouter::create_model_step(
                &settings.openrouter,
                &authentication.openrouter.issuer,
                &credentials_path.join("openrouter.json"),
                read_model_allowlist,
                read_models,
            );
            Ok(CreateModelStep::new(create))
        }
        _ => Err(format!("Replies are not supported for provider {provider_id}").into()),
    }
}
