mod callback;
mod discovery;
mod store;

use super::destination;
use super::endpoint;
use super::failure;
use crate::tool_io::Error;
use application::error_context::ResultExt;
use application::ports::PresentMcpAuthorization;
use discovery::Metadata;
use openidconnect::CsrfToken;
use openidconnect::PkceCodeChallenge;
use reqwest::Url;
use reqwest::header::HeaderMap;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use store::Grant;
use tokio_util::sync::CancellationToken;

pub(super) fn oauth_destination(value: &str) -> Result<Url, Error> {
    let url = destination(value)?;
    if url.query_pairs().any(|(key, _)| {
        matches!(
            key.to_ascii_lowercase().as_str(),
            "response_type"
                | "client_id"
                | "redirect_uri"
                | "state"
                | "code_challenge"
                | "code_challenge_method"
                | "resource"
                | "scope"
                | "grant_type"
                | "code"
                | "code_verifier"
                | "access_token"
                | "refresh_token"
                | "client_secret"
        )
    }) {
        return Err(failure(
            "MCP OAuth endpoint metadata must not preconfigure transaction or credential query parameters",
        ));
    }
    Ok(url)
}

fn metadata_id(value: &str) -> Result<Url, Error> {
    let url = destination(value)?;
    let raw_path = value
        .split_once("://")
        .and_then(|(_, authority)| authority.find('/').map(|start| &authority[start..]))
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");
    if url.scheme() != "https"
        || url.path() == "/"
        || raw_path.split('/').any(|part| {
            matches!(
                part.to_ascii_lowercase().replace("%2e", ".").as_str(),
                "." | ".."
            )
        })
    {
        return Err(failure(
            "MCP client metadata requires an already hosted HTTPS URL with a path and no dot segments; Harness does not provision hosting",
        ));
    }
    Ok(url)
}

pub(super) fn valid_scope(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 8192
        && value.bytes().any(|byte| byte != b' ')
        && value.bytes().all(|byte| {
            byte == b' '
                || byte == 0x21
                || (0x23..=0x5b).contains(&byte)
                || (0x5d..=0x7e).contains(&byte)
        })
}

pub(super) fn challenge_scope(headers: &HeaderMap) -> Result<Option<String>, Error> {
    let mut scope = None;
    let mut bytes = 0;
    for value in headers.get_all("WWW-Authenticate") {
        bytes += value.as_bytes().len();
        if bytes > 16384 {
            return Err(failure("MCP authentication challenges exceed 16 KiB"));
        }
        let mut parameters = discovery::parameters(
            value
                .to_str()
                .context("Invalid MCP authentication challenge header")?,
        )?;
        if let Some(value) = parameters.remove("scope") {
            if !valid_scope(&value) || scope.replace(value).is_some() {
                return Err(failure("Invalid or ambiguous MCP scope challenge"));
            }
        }
    }
    Ok(scope)
}

pub(super) fn retain_recovery(original: Error, recovery: Error) -> Error {
    RecoveryFailure { original, recovery }.into()
}

pub(super) fn validate(config: &domain::McpOauth) -> Result<(), Error> {
    endpoint(&config.issuer)?;
    match &config.client {
        domain::McpOauthClient::Dynamic => {}
        domain::McpOauthClient::Registered {
            client_id,
            redirect_uri,
            secret_reference,
        } => {
            if !store::secret(client_id)
                || client_id.len() > 8192
                || secret_reference.as_ref().is_some_and(|value| {
                    !value.starts_with("HARNESS_MCP_")
                        || value.len() > 64
                        || !value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                })
            {
                return Err(failure(
                    "MCP OAuth client ID must be bounded and client secrets must be provisioned HARNESS_MCP_ references",
                ));
            }
            callback::validate_redirect(redirect_uri)?;
        }
        domain::McpOauthClient::MetadataDocument {
            client_id,
            redirect_uri,
        } => {
            metadata_id(client_id)?;
            callback::validate_redirect(redirect_uri)?;
        }
    }
    Ok(())
}

fn now() -> Result<u64, Error> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

fn binding(
    grant: &Grant,
    resource: &Url,
    config: &domain::McpOauth,
    metadata: &Metadata,
) -> Result<(), Error> {
    if grant.resource != resource.as_str()
        || grant.issuer != config.issuer
        || grant.issuer != metadata.issuer
        || grant.token_endpoint != metadata.token
    {
        return Err(failure(
            "MCP credential resource/issuer/token endpoint changed; review configuration and use explicit Connect",
        ));
    }
    match &config.client {
        domain::McpOauthClient::Dynamic => {}
        domain::McpOauthClient::Registered {
            client_id,
            redirect_uri,
            ..
        }
        | domain::McpOauthClient::MetadataDocument {
            client_id,
            redirect_uri,
        } => {
            if &grant.client_id != client_id || &grant.redirect != redirect_uri {
                return Err(failure(
                    "MCP credential client or redirect identity changed; use explicit Connect",
                ));
            }
        }
    }
    Ok(())
}

fn secret(config: &domain::McpOauth, metadata: &Metadata) -> Result<Option<String>, Error> {
    if let domain::McpOauthClient::Registered {
        secret_reference: Some(reference),
        ..
    } = &config.client
    {
        if !metadata.secret_post_supported {
            return Err(failure(
                "This configured confidential MCP client requires advertised client_secret_post support; configure a compatible client",
            ));
        }
        let value =
            std::env::var(reference).context("MCP client secret reference is not provisioned")?;
        if !store::secret(&value) {
            return Err(failure("Invalid provisioned MCP client secret"));
        }
        return Ok(Some(value));
    }
    Ok(None)
}

async fn client_id(
    config: &domain::McpOauth,
    metadata: &Metadata,
    redirect: &str,
) -> Result<String, Error> {
    match &config.client {
        domain::McpOauthClient::Registered { client_id, .. } => Ok(client_id.clone()),
        domain::McpOauthClient::MetadataDocument { client_id, .. } => {
            if !metadata.document_supported {
                return Err(failure(
                    "MCP issuer does not advertise Client ID Metadata Documents; configure a pre-registered client or supported dynamic registration",
                ));
            }
            let response = discovery::client()?
                .get(metadata_id(client_id)?)
                .send()
                .await
                .context("MCP hosted client metadata is unavailable")?
                .error_for_status()
                .context("MCP hosted client metadata request failed")?;
            let document = discovery::json_document(response).await?;
            let client_id = client_id.clone();
            let redirect = redirect.to_owned();
            tokio::task::spawn_blocking(move || -> Result<String, Error> {
            if document.get("client_secret").is_some() || document.get("client_secret_expires_at").is_some() || document["client_id"].as_str() != Some(client_id.as_str())
                || document["client_name"].as_str().is_none_or(str::is_empty)
                || !document["redirect_uris"].as_array().is_some_and(|values| {
                    values.len() <= 128 && values.iter().any(|value| value.as_str() == Some(redirect.as_str()))
                })
                || document
                    .get("token_endpoint_auth_method")
                    .is_some_and(|value| value != "none")
            {
                return Err(failure(
                    "Hosted MCP client metadata does not bind this exact client/redirect or requires unsupported authentication",
                ));
            }
            Ok(client_id.clone())
            }).await?
        }
        domain::McpOauthClient::Dynamic => {
            let registration=metadata.registration.as_ref().ok_or_else(|| failure("MCP issuer requires client prerequisites: configure a pre-registered client or an already hosted Client ID Metadata Document; no public hosting is automatically provisioned"))?;
            let response=discovery::client()?.post(oauth_destination(registration)?).json(&json!({"client_name":"Harness", "application_type":"native", "redirect_uris":[redirect], "grant_types":["authorization_code","refresh_token"], "response_types":["code"], "token_endpoint_auth_method":"none"})).send().await.context("MCP client registration failed")?.error_for_status().context("MCP client registration was rejected; follow issuer prerequisites or configure a pre-registered client")?;
            let registration = discovery::json(response).await?;
            let redirect = redirect.to_owned();
            tokio::task::spawn_blocking(move || -> Result<String, Error> {
            let id = registration["client_id"]
                .as_str()
                .filter(|id| store::secret(id) && id.len() <= 8192)
                .ok_or_else(|| failure("MCP registration returned an invalid client ID"))?;
            if registration.get("client_secret").is_some()
                || registration
                    .get("token_endpoint_auth_method")
                    .is_some_and(|value| value != "none")
                || !registration["redirect_uris"]
                    .as_array()
                    .is_some_and(|values| values.len() == 1 && values[0] == redirect)
            {
                return Err(failure(
                    "MCP registration changed the redirect or requires unsupported confidential-client prerequisites; configure a compatible pre-registered client",
                ));
            }
            Ok(id.to_owned())
            }).await?
        }
    }
}

async fn token(
    metadata: Arc<Metadata>,
    form: Vec<(&str, String)>,
    previous: Option<Arc<Grant>>,
    rotating: bool,
) -> Result<Grant, Error> {
    let response=discovery::client()?.post(oauth_destination(&metadata.token)?).form(&form).send().await.context("MCP token exchange failed; use explicit Connect rather than retrying an uncertain exchange")?.error_for_status().context("MCP token exchange was rejected; use explicit Connect to recover authorization")?;
    let response = discovery::json(response).await?;
    let decoding_metadata = Arc::clone(&metadata);
    let decoding_previous = previous.clone();
    let (mut grant, identity_expected) =
        tokio::task::spawn_blocking(move || -> Result<_, Error> {
            let metadata = decoding_metadata;
            let previous = decoding_previous.as_deref();
            if response["token_type"]
                .as_str()
                .is_none_or(|value| !value.eq_ignore_ascii_case("Bearer"))
            {
                return Err(failure("MCP OAuth requires Bearer tokens"));
            }
            let access = response["access_token"]
                .as_str()
                .filter(|value| store::access(value))
                .ok_or_else(|| failure("MCP OAuth returned an invalid access token"))?
                .to_owned();
            let refresh = response
                .get("refresh_token")
                .map(|value| {
                    value
                        .as_str()
                        .filter(|value| store::secret(value))
                        .map(str::to_owned)
                        .ok_or_else(|| failure("MCP OAuth returned an invalid refresh token"))
                })
                .transpose()?
                .or_else(|| {
                    if rotating {
                        previous.and_then(|grant| grant.refresh.clone())
                    } else {
                        None
                    }
                });
            let expires = response["expires_in"]
                .as_u64()
                .filter(|seconds| *seconds > 0 && *seconds <= 31536000)
                .ok_or_else(|| failure("MCP OAuth requires a bounded access-token expiry"))?;
            let scope = response
                .get("scope")
                .map(|scope| {
                    scope
                        .as_str()
                        .filter(|scope| valid_scope(scope))
                        .map(str::to_owned)
                        .ok_or_else(|| failure("Invalid MCP granted scope"))
                })
                .transpose()?
                .or_else(|| {
                    if rotating {
                        previous.and_then(|grant| grant.scope.clone())
                    } else {
                        metadata.scope.clone()
                    }
                });
            if let Some(required) = &metadata.scope {
                if !required.split_ascii_whitespace().all(|required| {
                    scope.as_ref().is_some_and(|granted| {
                        granted
                            .split_ascii_whitespace()
                            .any(|granted| granted == required)
                    })
                }) {
                    return Err(failure(
                        "MCP OAuth did not grant the required resource scopes",
                    ));
                }
            }
            let identity_token = response
                .get("id_token")
                .map(|value| {
                    value
                        .as_str()
                        .filter(|value| store::secret(value))
                        .map(str::to_owned)
                        .ok_or_else(|| failure("MCP issuer returned an invalid identity token"))
                })
                .transpose()?
                .or_else(|| {
                    if rotating {
                        previous.and_then(|grant| grant.identity_token.clone())
                    } else {
                        None
                    }
                });
            let identity_expected = scope.as_ref().is_some_and(|scope| {
                scope
                    .split_ascii_whitespace()
                    .any(|scope| scope == "openid")
            }) || response.get("id_token").is_some()
                || previous.is_some_and(|grant| grant.account.is_some());

            let grant = Grant {
                generation: 0,
                resource: String::new(),
                issuer: metadata.issuer.clone(),
                client_id: String::new(),
                redirect: String::new(),
                token_endpoint: metadata.token.clone(),
                account: None,
                access,
                refresh,
                identity_token,
                expires_at: now()?
                    .checked_add(expires)
                    .ok_or_else(|| failure("MCP token expiry overflow"))?,
                scope,
                required_scope: None,
                unusable: false,
            };
            Ok((grant, identity_expected))
        })
        .await??;
    let account = if identity_expected {
        let userinfo = metadata.userinfo.as_ref().ok_or_else(|| failure("MCP identity-bearing grants require supported account verification; configure a verifiable userinfo endpoint or a non-OIDC resource grant"))?;
        let response = discovery::client()?
            .get(oauth_destination(userinfo)?)
            .bearer_auth(&grant.access)
            .send()
            .await
            .context("MCP account verification failed")?
            .error_for_status()
            .context("MCP issuer rejected account verification")?;
        let identity = discovery::json(response).await?;
        Some(
            tokio::task::spawn_blocking(move || -> Result<String, Error> {
                Ok(identity["sub"]
                    .as_str()
                    .filter(|value| store::secret(value) && value.len() <= 8192)
                    .ok_or_else(|| {
                        failure("MCP account verification did not return a valid subject")
                    })?
                    .to_owned())
            })
            .await??,
        )
    } else {
        None
    };
    if let Some(previous) = previous {
        if previous.account != account {
            return Err(failure(
                "MCP authorization account changed; disconnect and explicitly review a new connection before replacing its identity",
            ));
        }
    }

    grant.account = account;
    Ok(grant)
}

struct RecoveryFailure {
    original: Error,
    recovery: Error,
}

impl std::fmt::Debug for RecoveryFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecoveryFailure")
            .field("original", &self.original)
            .field("recovery", &self.recovery)
            .finish()
    }
}

impl std::fmt::Display for RecoveryFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MCP authentication failed and private state recovery also failed; preserve the record and reload")
    }
}
impl std::error::Error for RecoveryFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.original.as_ref())
    }
}

async fn finish_failure(path: &Path, pending: &str, original: Error) -> Error {
    match store::abandon(path, pending).await {
        Ok(()) => original,
        Err(recovery) => RecoveryFailure { original, recovery }.into(),
    }
}

pub(crate) struct Authorization {
    path: PathBuf,
    pending: String,
    previous: Option<Grant>,
}

pub(super) async fn admit(
    path: &Path,
    cancellation: CancellationToken,
) -> Result<Authorization, Error> {
    let (pending, previous, _) = store::begin(path, false, None, None, cancellation).await?;
    Ok(Authorization {
        path: path.to_owned(),
        pending,
        previous,
    })
}

pub(super) async fn authenticate(
    attempt: Authorization,
    resource: &Url,
    config: &domain::McpOauth,
    present: &PresentMcpAuthorization,
    cancellation: CancellationToken,
) -> Result<(), Error> {
    let Authorization {
        path,
        pending,
        previous,
    } = attempt;
    let path = path.as_path();
    let operation = async {
    validate(config)?;
    let mut metadata = tokio::select! { biased; _ = cancellation.cancelled() => return Err(failure("MCP authorization cancelled before discovery")), metadata=discovery::discover(resource, &config.issuer) => metadata? };
    if let Some(previous) = &previous {
        if previous.resource != resource.as_str() || previous.issuer != config.issuer { return Err(failure("Saved MCP authorization identity does not match this explicit Connect")); }
        let mut scopes = BTreeSet::new();
        for scope in [previous.scope.as_deref(), previous.required_scope.as_deref(), metadata.scope.as_deref()].into_iter().flatten() { scopes.extend(scope.split_ascii_whitespace()); }
        let scope = scopes.into_iter().collect::<Vec<_>>().join(" ");
        if scope.len() > 8192 { return Err(failure("Combined MCP authorization scopes exceed their bound")); }
        if !scope.is_empty() { metadata.scope = Some(scope); }
    }
    let redirect = match &config.client {
        domain::McpOauthClient::Dynamic => None,
        domain::McpOauthClient::Registered { redirect_uri, .. }
        | domain::McpOauthClient::MetadataDocument { redirect_uri, .. } => {
            Some(redirect_uri.as_str())
        }
    };
    let (listener, redirect) = callback::bind(redirect).await?;
    let client_id = tokio::select! { biased; _ = cancellation.cancelled() => return Err(failure("MCP authorization cancelled before client setup")), client_id=client_id(config, &metadata, &redirect) => client_id? };
    let client_secret = secret(config, &metadata)?;
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let state = CsrfToken::new_random();
    let mut callback = callback::start(
        listener,
        state.secret(),
        &metadata.issuer,
        metadata.iss_required,
        &client_id,
        resource.as_str(),
    )
    .await?;
    let mut authorization = oauth_destination(&metadata.authorization)?;
    authorization
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect)
        .append_pair("state", state.secret())
        .append_pair("code_challenge", challenge.as_str())
        .append_pair("code_challenge_method", "S256")
        .append_pair("resource", resource.as_str());
    if let Some(scope) = &metadata.scope {
        authorization.query_pairs_mut().append_pair("scope", scope);
    }
    if authorization.as_str().len() > 65536 { return Err(failure("MCP authorization URL exceeds 64 KiB; configure shorter identifiers or scopes")); }
    if cancellation.is_cancelled() {
        return Err(failure("MCP authorization cancelled before browser admission"));
    }

        if cancellation.is_cancelled() { return Err::<(), Error>("MCP authorization cancelled before browser presentation".into()); }
        present.call(authorization.into()).map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;
        let code=tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err::<(), Error>("MCP authorization cancelled; no code exchanged".into()),
            result=tokio::time::timeout(Duration::from_secs(600), callback.receiver.recv()) => result.context("MCP authorization callback timed out")?.ok_or_else(|| failure("MCP callback listener closed"))??,
        };
        if cancellation.is_cancelled() { return Err(failure("MCP authorization cancelled before token exchange")); }
        let mut form=vec![("grant_type", "authorization_code".into()), ("code", code), ("client_id", client_id.clone()), ("redirect_uri", redirect.clone()), ("code_verifier", verifier.secret().to_owned()), ("resource", resource.to_string())];
        if let Some(secret)=client_secret { form.push(("client_secret", secret)); }
        let mut grant=token(Arc::new(metadata), form, previous.map(Arc::new), false).await?;
        grant.resource=resource.to_string(); grant.client_id=client_id; grant.redirect=redirect;
        store::complete(path, &pending, grant).await
    }.await;
    match operation {
        Ok(()) => Ok(()),
        Err(error) => Err(finish_failure(path, &pending, error).await),
    }
}

pub(super) async fn resolve(
    path: &Path,
    resource: &Url,
    config: &domain::McpOauth,
    cancellation: CancellationToken,
) -> Result<(String, Vec<String>, u64), Error> {
    let metadata = tokio::select! { biased; _ = cancellation.cancelled() => return Err(failure("MCP credential resolution cancelled")), metadata=discovery::discover(resource, &config.issuer) => metadata? };
    let record = store::load(path).await?;
    let generation = record.generation;
    let metadata = Arc::new(metadata);
    let mut grant = record
        .grant
        .ok_or_else(|| failure("MCP OAuth is not connected; use explicit native /mcp Connect"))?;
    binding(&grant, resource, config, &metadata)?;
    if grant.unusable {
        return Err(failure(
            "MCP OAuth grant requires explicit Connect; an earlier rotating exchange or revocation may have invalidated it",
        ));
    }
    if grant.expires_at <= now()?.saturating_add(30) {
        let refresh = grant.refresh.clone().ok_or_else(|| {
            failure("MCP OAuth token expired without a refresh token; use explicit Connect")
        })?;
        let client_secret = secret(config, &metadata)?;
        if cancellation.is_cancelled() {
            return Err(failure("MCP refresh cancelled before admission"));
        }
        let (pending, previous, refresh_generation) = store::begin(
            path,
            true,
            Some(&grant.access),
            Some(generation),
            cancellation.clone(),
        )
        .await?;
        if cancellation.is_cancelled() {
            let original = failure("MCP refresh cancelled before token exchange");
            let recovery = match previous {
                Some(previous) => store::complete(path, &pending, previous).await,
                None => Err(failure("MCP refresh recovery lost its prior grant")),
            };
            return Err(match recovery {
                Ok(()) => original,
                Err(recovery) => retain_recovery(original, recovery),
            });
        }
        let mut form = vec![
            ("grant_type", "refresh_token".into()),
            ("refresh_token", refresh),
            ("client_id", grant.client_id.clone()),
            ("resource", resource.to_string()),
        ];
        if let Some(secret) = client_secret {
            form.push(("client_secret", secret));
        }
        let (current, previous) = tokio::task::spawn_blocking(move || {
            let previous = Arc::new(grant.clone());
            (grant, previous)
        })
        .await?;
        grant = current;
        let replacement = token(Arc::clone(&metadata), form, Some(previous), true).await;
        let mut replacement = match replacement {
            Ok(replacement) => replacement,
            Err(error) => return Err(finish_failure(path, &pending, error).await),
        };
        replacement.resource = grant.resource;
        replacement.client_id = grant.client_id;
        replacement.redirect = grant.redirect;
        replacement.generation = refresh_generation;
        let (replacement, saved) = tokio::task::spawn_blocking(move || {
            let saved = replacement.clone();
            (replacement, saved)
        })
        .await?;
        store::complete(path, &pending, saved).await?;
        grant = replacement;
    }
    let mut secrets = vec![grant.access.clone()];
    if let Some(refresh) = grant.refresh {
        secrets.push(refresh);
    }
    if let Some(identity_token) = grant.identity_token {
        secrets.push(identity_token);
    }
    if let Some(account) = grant.account {
        secrets.push(account);
    }
    if let Some(secret) = secret(config, &metadata)? {
        secrets.push(secret);
    }
    Ok((grant.access, secrets, grant.generation))
}

pub(super) async fn invalidate(
    path: &Path,
    access: &str,
    generation: u64,
    required_scope: Option<String>,
) -> Result<(), Error> {
    let _lock = store::lock(path, &CancellationToken::new()).await?;
    let mut record = store::load(path).await?;
    if let Some(grant) = &mut record.grant {
        if grant.generation == generation && grant.access == access {
            grant.unusable = true;
            grant.required_scope = required_scope;
            store::save(path, record).await?;
        }
    }
    Ok(())
}

pub(super) async fn disconnect(path: &Path) -> Result<(), Error> {
    let _lock = store::lock(path, &CancellationToken::new()).await?;
    let mut record = store::load(path).await?;
    record.pending = None;
    record.grant = None;
    record.generation = record
        .generation
        .checked_add(1)
        .ok_or_else(|| failure("MCP credential generation exhausted"))?;
    store::save(path, record).await
}
