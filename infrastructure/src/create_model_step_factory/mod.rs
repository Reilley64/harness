use std::path::Path;

use application::ports::CreateModelStep;

use crate::create_model_step::ModelStepError;
use crate::create_model_step::Settings;
use crate::create_model_step::openai;
use crate::create_provider_connection::Settings as AuthenticationSettings;

pub fn create_model_step_factory(
    settings: &Settings,
    authentication: &AuthenticationSettings,
    credentials_path: &Path,
    provider_id: &domain::ProviderId,
    read_model_allowlist: application::ports::ReadModelAllowlist,
) -> Result<CreateModelStep, ModelStepError> {
    match provider_id.0.as_str() {
        "openai" => {
            let create = openai::create_model_step(
                &settings.openai,
                &authentication.openai,
                &credentials_path.join("openai.json"),
                read_model_allowlist,
            );
            Ok(CreateModelStep::new(create))
        }
        _ => Err(format!("Replies are not supported for provider {provider_id}").into()),
    }
}
