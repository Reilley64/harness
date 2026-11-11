mod callback;
pub(crate) mod openai;
pub(crate) mod openrouter;
pub(crate) mod settings;
pub(crate) mod storage;

pub use settings::Settings;

pub(crate) type AuthenticationError = Box<dyn std::error::Error + Send + Sync>;
