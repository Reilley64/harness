mod context;
mod credentials;
pub(crate) mod openai;
mod record;
mod stream;
mod usage;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub openai: OpenaiSettings,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenaiSettings {
    pub base_url: String,
    pub timeout_seconds: u64,
}

pub(crate) type ModelStepError = Box<dyn std::error::Error + Send + Sync>;
pub type ModelStepFuture = application::ports::PortFuture<
    Result<domain::ModelStep, application::ports::CreateModelStepError>,
>;

pub(crate) fn checked_url(value: &str) -> Result<reqwest::Url, ModelStepError> {
    let url = reqwest::Url::parse(value)?;
    let loopback = url
        .host_str()
        .is_some_and(|host| host == "127.0.0.1" || host == "[::1]");
    if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Provider URLs must use HTTPS, or loopback HTTP for fixtures, without credentials/query/fragment".into());
    }
    Ok(url)
}
