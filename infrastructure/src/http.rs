use openidconnect::HttpClientError;
use openidconnect::HttpRequest;
use openidconnect::HttpResponse;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::Pin;

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn body(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>, Error> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err("Provider response exceeds its size limit".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            return Err("Provider response exceeds its size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub(crate) async fn json<T: DeserializeOwned + Send + 'static>(
    response: reqwest::Response,
    limit: usize,
) -> Result<T, Error> {
    let bytes = body(response, limit).await?;
    tokio::task::spawn_blocking(move || Ok(serde_json::from_slice(&bytes)?)).await?
}

pub(crate) struct IdentityClient(pub(crate) reqwest::Client);

impl<'a> openidconnect::AsyncHttpClient<'a> for IdentityClient {
    type Error = HttpClientError<reqwest::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + 'a>>;

    fn call(&'a self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            let client = &self.0;
            let request: reqwest::Request = request.try_into().map_err(Box::new)?;
            let url = request.url();
            let loopback = url
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "[::1]"));
            if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err(HttpClientError::Other(
                    "Identity endpoints must use HTTPS without URL credentials".to_owned(),
                ));
            }
            let response = client.execute(request).await.map_err(Box::new)?;
            let mut builder = openidconnect::http::Response::builder()
                .status(response.status())
                .version(response.version());
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            let bytes = body(response, 1_048_576)
                .await
                .map_err(|error| HttpClientError::Io(std::io::Error::other(error)))?;
            Ok(builder.body(bytes)?)
        })
    }
}
