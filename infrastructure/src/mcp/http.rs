mod auth;
mod decoder;

pub(super) use auth::Authorization;

use crate::tool_io::Error;
use application::error_context::ResultExt;
use application::ports::PresentMcpAuthorization;
use decoder::Decoder;
use reqwest::Client;
use reqwest::Response;
use reqwest::Url;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const LIMIT: usize = 1_048_576;

pub(super) fn failure(message: &'static str) -> Error {
    match Err::<(), _>(message).context(message) {
        Err(error) => error.into(),
        Ok(()) => message.into(),
    }
}

pub(super) struct Transport {
    client: Client,
    auth_path: PathBuf,
    oauth: Option<domain::McpOauth>,
    url: Url,
    headers: HeaderMap,
    session: Option<HeaderValue>,
    protocol: Option<HeaderValue>,
    incoming: Option<Incoming>,
    lost: bool,
    call_sent: bool,
    pending_revocation: bool,
    required_scope: Option<String>,
    grant_generation: Option<u64>,
}

struct Incoming {
    response: Response,
    sse: bool,
    decoder: Decoder,
}

pub(super) fn destination(value: &str) -> Result<Url, Error> {
    if value.len() > 8192 {
        return Err(failure("MCP HTTP destination exceeds 8 KiB"));
    }
    let url = Url::parse(value).context("Invalid MCP HTTP destination")?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || (url.scheme() != "https"
            && !(url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"))))
    {
        return Err(failure(
            "MCP HTTP destinations require HTTPS (or explicit loopback), without credentials or fragment",
        ));
    }
    Ok(url)
}

pub(super) fn endpoint(value: &str) -> Result<Url, Error> {
    let url = destination(value)?;
    if url.query().is_some() {
        return Err(failure(
            "MCP resource and issuer URLs must not include a query",
        ));
    }
    Ok(url)
}

pub(super) async fn admit(
    path: &Path,
    cancellation: CancellationToken,
) -> Result<Authorization, Error> {
    auth::admit(path, cancellation).await
}

pub(super) async fn authenticate(
    attempt: Authorization,
    url: &str,
    config: &domain::McpOauth,
    present: &PresentMcpAuthorization,
    cancellation: CancellationToken,
) -> Result<(), Error> {
    auth::authenticate(attempt, &endpoint(url)?, config, present, cancellation).await
}

pub(super) fn validate_oauth(config: &domain::McpOauth) -> Result<(), Error> {
    auth::validate(config)
}

pub(super) async fn disconnect(path: &Path) -> Result<(), Error> {
    auth::disconnect(path).await
}

pub(super) fn reserved_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "host"
            | "accept"
            | "content-type"
            | "content-length"
            | "connection"
            | "transfer-encoding"
            | "cookie"
            | "origin"
            | "mcp-session-id"
            | "mcp-protocol-version"
            | "last-event-id"
    )
}

impl Transport {
    pub(super) fn start(
        auth_path: &Path,
        url: &str,
        references: &BTreeMap<String, String>,
        oauth: Option<&domain::McpOauth>,
    ) -> Result<(Self, Vec<String>), Error> {
        let url = endpoint(url)?;
        if oauth.is_some()
            && references
                .keys()
                .any(|name| name.eq_ignore_ascii_case("authorization"))
        {
            return Err(failure(
                "MCP OAuth cannot be combined with a provisioned Authorization header",
            ));
        }
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .timeout(Duration::from_secs(120))
            .build()?;
        let mut headers = HeaderMap::new();
        let mut secrets = Vec::new();
        for (name, reference) in references {
            if reserved_header(name) || !reference.starts_with("HARNESS_MCP_") {
                return Err(failure(
                    "MCP header is reserved or does not reference a provisioned HARNESS_MCP_ value",
                ));
            }
            let secret = std::env::var(reference)
                .context("Required MCP header reference is not provisioned")?;
            if secret.is_empty() || secret.len() > 8192 {
                return Err(failure("MCP provisioned header is empty or too long"));
            }
            let name =
                HeaderName::from_bytes(name.as_bytes()).context("Invalid MCP header name")?;
            let mut value =
                HeaderValue::from_str(&secret).context("Invalid MCP provisioned header")?;
            value.set_sensitive(true);
            if name == "authorization" {
                if let Some((scheme, credential)) = secret.split_once(' ') {
                    let credential = credential.trim();
                    if (scheme.eq_ignore_ascii_case("Bearer")
                        || scheme.eq_ignore_ascii_case("Basic"))
                        && !credential.is_empty()
                    {
                        secrets.push(credential.to_owned());
                    }
                }
            }
            if headers.insert(name, value).is_some() {
                return Err(failure("Duplicate case-insensitive MCP header"));
            }
            secrets.push(secret);
        }
        Ok((
            Self {
                client,
                auth_path: auth_path.to_owned(),
                oauth: oauth.cloned(),
                url,
                headers,
                session: None,
                protocol: None,
                incoming: None,
                lost: false,
                call_sent: false,
                pending_revocation: false,
                required_scope: None,
                grant_generation: None,
            },
            secrets,
        ))
    }

    pub(super) async fn prepare(
        &mut self,
        cancellation: CancellationToken,
    ) -> Result<Vec<String>, Error> {
        self.flush_revocation().await?;
        if let Some(config) = &self.oauth {
            let (token, secrets, generation) =
                auth::resolve(&self.auth_path, &self.url, config, cancellation).await?;
            let mut value = HeaderValue::from_str(&format!("Bearer {token}"))
                .context("Invalid MCP access token header")?;
            value.set_sensitive(true);
            self.headers.insert("Authorization", value);
            self.grant_generation = Some(generation);
            return Ok(secrets);
        }
        Ok(Vec::new())
    }

    pub(super) fn negotiate(&mut self, version: &str) -> Result<(), Error> {
        if version != "2025-06-18" {
            return Err(failure(
                "HTTP MCP supports protocol 2025-06-18; legacy SSE is not supported",
            ));
        }
        self.protocol = Some(HeaderValue::from_static("2025-06-18"));
        self.lost = false;
        Ok(())
    }

    pub(super) fn session_identity(&self) -> Option<String> {
        self.session
            .as_ref()
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    }

    pub(super) fn session_lost(&self) -> bool {
        self.lost
    }

    fn request(&self, method: reqwest::Method) -> reqwest::RequestBuilder {
        let mut request = self
            .client
            .request(method, self.url.clone())
            .headers(self.headers.clone())
            .header("Accept", "application/json, text/event-stream");
        if let Some(session) = &self.session {
            request = request.header("Mcp-Session-Id", session.clone());
        }
        if let Some(protocol) = &self.protocol {
            request = request.header("MCP-Protocol-Version", protocol.clone());
        }
        request
    }

    pub(super) fn call_sent(&self) -> bool {
        self.call_sent
    }

    pub(super) fn reset_call_sent(&mut self) {
        self.call_sent = false;
    }

    pub(super) async fn send(
        &mut self,
        bytes: Vec<u8>,
        request: bool,
        tool_call: bool,
    ) -> Result<(), Error> {
        self.call_sent |= tool_call;
        let response = self
            .request(reqwest::Method::POST)
            .header("Content-Type", "application/json")
            .body(bytes)
            .send()
            .await
            .context("MCP HTTP request failed")?;
        match response.status().as_u16() {
            401 | 403 => {
                self.pending_revocation = self.oauth.is_some();
                let scope = auth::challenge_scope(response.headers());
                let original: Error = match response.error_for_status() {
                    Err(error) => error.into(),
                    Ok(_) => "MCP authorization was rejected".into(),
                };
                let original = match scope {
                    Ok(scope) => {
                        self.required_scope = scope;
                        original
                    }
                    Err(recovery) => auth::retain_recovery(original, recovery),
                };
                return Err(original).context("MCP authorization is missing, expired, revoked or lacks scope. Configure transport.oauth issuer/client prerequisites or correct provisioned header references, then use explicit /mcp Connect. No request is replayed").map_err(Into::into);
            }
            404 if self.session.is_some() => {
                self.session = None;
                self.protocol = None;
                self.lost = true;
                self.incoming = None;
                return response.error_for_status().map(|_| ()).context("MCP session expired; a new session is required. A sent tool call is not replayed").map_err(Into::into);
            }
            _ => {}
        }
        let response = response
            .error_for_status()
            .context("MCP HTTP server rejected the request")?;
        if !request {
            if response.status().as_u16() != 202
                || response.content_length().is_some_and(|length| length != 0)
            {
                return Err(failure(
                    "MCP notification/response requires an empty HTTP 202 response",
                ));
            }
            let mut response = response;
            for _ in 0..256 {
                match response
                    .chunk()
                    .await
                    .context("MCP HTTP acknowledgement was interrupted")?
                {
                    None => return Ok(()),
                    Some(chunk) if chunk.is_empty() => {}
                    Some(_) => {
                        return Err(failure("MCP HTTP 202 response must not contain a body"));
                    }
                }
            }
            return Err(failure("MCP HTTP acknowledgement exceeded its frame bound"));
        }
        if response.status().as_u16() != 200 {
            return Err(failure("MCP HTTP request requires a JSON or SSE response"));
        }
        let sessions = response.headers().get_all("Mcp-Session-Id");
        if sessions.iter().count() > 1 {
            return Err(failure("Duplicate MCP session identity"));
        }
        if let Some(session) = sessions.iter().next() {
            let value = session.as_bytes();
            if value.is_empty()
                || value.len() > 1024
                || !value.iter().all(|byte| (0x21..=0x7e).contains(byte))
            {
                return Err(failure("Invalid MCP session identity"));
            }
            if self.protocol.is_none() {
                let mut session = session.clone();
                session.set_sensitive(true);
                self.session = Some(session);
            } else if self.session.as_ref() != Some(session) {
                return Err(failure(
                    "MCP server changed session identity outside initialization",
                ));
            }
        }
        if response.headers().get_all("Content-Type").iter().count() != 1 {
            return Err(failure(
                "MCP HTTP response requires exactly one content type",
            ));
        }
        let content = response
            .headers()
            .get("Content-Type")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::trim)
            .ok_or_else(|| failure("Missing MCP HTTP content type"))?;
        let sse = if content.eq_ignore_ascii_case("text/event-stream") {
            true
        } else if content.eq_ignore_ascii_case("application/json") {
            false
        } else {
            return Err(failure(
                "Unsupported MCP HTTP content type; legacy SSE is not supported",
            ));
        };
        self.incoming = Some(Incoming {
            response,
            sse,
            decoder: Decoder::default(),
        });
        Ok(())
    }

    pub(super) fn finish_response(&mut self) {
        self.incoming = None;
    }

    pub(super) async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let incoming = self
            .incoming
            .as_mut()
            .ok_or_else(|| failure("Missing MCP HTTP response"))?;
        incoming.receive().await
    }

    async fn flush_revocation(&mut self) -> Result<(), Error> {
        if self.pending_revocation {
            let access = self
                .headers
                .get("Authorization")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .ok_or_else(|| failure("Missing rejected MCP credential identity"))?;
            let generation = self
                .grant_generation
                .ok_or_else(|| failure("Missing rejected MCP credential generation"))?;
            auth::invalidate(
                &self.auth_path,
                access,
                generation,
                self.required_scope.clone(),
            )
            .await?;
            self.required_scope = None;
            self.pending_revocation = false;
        }
        Ok(())
    }

    pub(super) async fn close(&mut self) -> Result<(), Error> {
        self.incoming = None;
        self.flush_revocation().await
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        if self.session.is_none() {
            return;
        }
        let request = self.request(reqwest::Method::DELETE);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let _ = request.timeout(Duration::from_secs(5)).send().await;
            });
        }
    }
}

impl Incoming {
    async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        if !self.sse {
            let mut bytes = Vec::new();
            while let Some(chunk) = self
                .response
                .chunk()
                .await
                .context("MCP HTTP response interrupted")?
            {
                if bytes.len() + chunk.len() > LIMIT {
                    return Err(failure("MCP JSON frame exceeds 1 MiB"));
                }
                bytes.extend_from_slice(&chunk);
            }
            return Ok(bytes);
        }
        loop {
            let mut decoder = std::mem::take(&mut self.decoder);
            let (decoder, frame) = tokio::task::spawn_blocking(move || -> Result<_, Error> {
                let frame = decoder.next()?;
                Ok((decoder, frame))
            })
            .await??;
            self.decoder = decoder;
            if let Some(frame) = frame {
                return Ok(frame);
            }

            let chunk = self
                .response
                .chunk()
                .await
                .context("MCP SSE response interrupted")?
                .ok_or_else(|| failure("MCP SSE closed before a complete correlated response"))?;
            let mut decoder = std::mem::take(&mut self.decoder);
            self.decoder = tokio::task::spawn_blocking(move || -> Result<_, Error> {
                decoder.append(&chunk)?;
                Ok(decoder)
            })
            .await??;
        }
    }
}
