use crate::cache::remote_json;
use crate::cache::with_file_fallback;
use crate::cache::with_memory_cache;
use application::error_context::ResultExt;
use application::ports::PortFuture;
use std::path::Path;
mod response;
mod settings;

use self::response::Response;
pub use settings::Settings;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

type ReadModelsFuture = Pin<
    Box<
        dyn Future<Output = Result<Vec<Response>, Box<dyn std::error::Error + Send + Sync>>> + Send,
    >,
>;

fn remote(url: &str) -> impl Fn() -> ReadModelsFuture + Send + Sync + use<> {
    let url = url.to_owned();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("HTTP client initialization failed");
    move || {
        let client = client.clone();
        let url = url.clone();
        Box::pin(remote_json(client, url))
    }
}

pub fn read_models(
    url: &str,
    path: &Path,
    ttl: Duration,
) -> impl Fn() -> PortFuture<Result<Vec<domain::Model>, Box<dyn std::error::Error + Send + Sync>>>
+ Send
+ Sync
+ use<> {
    let cached = with_memory_cache(with_file_fallback(remote(url), path), ttl);
    move || {
        let future = cached();
        Box::pin(async move {
            let responses = future.await.context("Could not load model catalog")?;
            tokio::task::spawn_blocking(move || {
                let models: Vec<domain::Model> = responses.into_iter().map(Into::into).collect();
                let mut ids = std::collections::HashSet::new();
                if models.iter().any(|model| {
                    model.id.is_empty()
                        || !ids.insert((model.provider_id.clone(), model.id.clone()))
                }) {
                    return Err("Invalid or duplicate catalog model identity".into());
                }
                Ok(models)
            })
            .await
            .context("Could not process model catalog")?
        })
    }
}
