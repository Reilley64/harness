use super::super::destination;
use super::super::endpoint;
use super::super::failure;
use super::oauth_destination;
use super::valid_scope;
use crate::tool_io::Error;
use application::error_context::ResultExt;
use reqwest::Client;
use reqwest::Response;
use reqwest::Url;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

pub(super) struct Metadata {
    pub issuer: String,
    pub authorization: String,
    pub token: String,
    pub registration: Option<String>,
    pub userinfo: Option<String>,
    pub scope: Option<String>,
    pub iss_required: bool,
    pub document_supported: bool,
    pub secret_post_supported: bool,
}

pub(super) fn client() -> Result<Client, Error> {
    Ok(Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(Duration::from_secs(15))
        .build()?)
}

pub(super) async fn json(response: Response) -> Result<Value, Error> {
    body(response, false).await
}

pub(super) async fn json_document(response: Response) -> Result<Value, Error> {
    body(response, true).await
}

async fn body(mut response: Response, document: bool) -> Result<Value, Error> {
    if response.headers().get_all("Content-Type").iter().count() != 1 {
        return Err(failure(
            "MCP authentication responses require exactly one JSON content type",
        ));
    }
    let content = response
        .headers()
        .get("Content-Type")
        .ok_or_else(|| failure("Missing MCP authentication content type"))?
        .to_str()
        .context("Invalid MCP authentication content type")?
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if content != "application/json"
        && !(document && content.starts_with("application/") && content.ends_with("+json"))
    {
        return Err(failure(
            "MCP authentication requires application/json; client metadata documents may use application/*+json",
        ));
    }
    let mut bytes = Vec::new();
    if response
        .content_length()
        .is_some_and(|length| length > 1_048_576)
    {
        return Err(failure("MCP authentication metadata exceeds 1 MiB"));
    }
    while let Some(chunk) = response
        .chunk()
        .await
        .context("MCP authentication response interrupted")?
    {
        if bytes.len() + chunk.len() > 1_048_576 {
            return Err(failure("MCP authentication response exceeds 1 MiB"));
        }
        bytes.extend_from_slice(&chunk);
    }
    tokio::task::spawn_blocking(move || -> Result<Value, Error> {
        Ok(serde_json::from_slice(&bytes).context("Invalid MCP authentication JSON")?)
    })
    .await?
}

#[derive(Debug)]
struct DiscoveryFailures(Vec<Error>);

impl std::fmt::Display for DiscoveryFailures {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MCP metadata discovery failed; configure the server's documented issuer/client prerequisites")
    }
}

impl std::error::Error for DiscoveryFailures {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0
            .last()
            .map(|error| error.as_ref() as &(dyn std::error::Error + 'static))
    }
}

async fn metadata(client: &Client, urls: Vec<Url>) -> Result<Value, Error> {
    let mut failures: Vec<Error> = Vec::new();
    for url in urls {
        let response = match client
            .get(url)
            .send()
            .await
            .context("MCP metadata discovery failed")
        {
            Ok(response) => response,
            Err(error) => {
                failures.push(error.into());
                return Err(DiscoveryFailures(failures).into());
            }
        };
        let fallback = matches!(response.status().as_u16(), 404 | 405);
        let response = match response
            .error_for_status()
            .context("MCP metadata endpoint rejected discovery")
        {
            Ok(response) => response,
            Err(error) => {
                failures.push(error.into());
                if fallback {
                    continue;
                }
                return Err(DiscoveryFailures(failures).into());
            }
        };
        if response.status().as_u16() != 200 {
            failures.push("MCP metadata endpoints require HTTP 200 without redirects".into());
            return Err(DiscoveryFailures(failures).into());
        }
        match json(response).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                failures.push(error);
                return Err(DiscoveryFailures(failures).into());
            }
        }
    }
    Err(DiscoveryFailures(failures).into())
}

fn well_known(base: &Url, suffix: &str, append: bool) -> Url {
    let mut url = base.clone();
    let path = if append {
        format!("{}/.well-known/{suffix}", base.path().trim_end_matches('/'))
    } else {
        format!("/.well-known/{suffix}{}", base.path().trim_end_matches('/'))
    };
    url.set_path(&path);
    url
}

pub(super) async fn discover(resource: &Url, issuer: &str) -> Result<Metadata, Error> {
    let client = client()?;
    let response = client
        .get(resource.clone())
        .header("Accept", "text/event-stream")
        .header("MCP-Protocol-Version", "2025-06-18")
        .send()
        .await
        .context("MCP authentication challenge request failed")?;
    if !matches!(response.status().as_u16(), 200 | 401 | 403 | 405) {
        return Err(failure(
            "MCP server did not provide a usable authorization challenge",
        ));
    }
    let mut headers = Vec::new();
    let mut header_bytes = 0;
    for value in response.headers().get_all("WWW-Authenticate") {
        header_bytes += value.as_bytes().len();
        if header_bytes > 16384 {
            return Err(failure("MCP authentication challenges exceed 16 KiB"));
        }
        headers.push(
            value
                .to_str()
                .context("Invalid MCP authentication challenge header")?
                .to_owned(),
        );
    }
    let challenge = tokio::task::spawn_blocking(move || -> Result<_, Error> {
        let mut challenge = BTreeMap::new();
        for value in headers {
            for (key, value) in parameters(&value)? {
                if challenge.insert(key, value).is_some() {
                    return Err(failure("Ambiguous MCP Bearer challenge"));
                }
            }
        }
        Ok(challenge)
    })
    .await??;
    drop(response);
    let urls = if let Some(value) = challenge.get("resource_metadata") {
        let url = destination(value)?;
        if url.origin() != resource.origin() {
            return Err(failure(
                "MCP protected-resource metadata must belong to the configured resource origin",
            ));
        }
        vec![url]
    } else {
        let path = well_known(resource, "oauth-protected-resource", false);
        let mut root = resource.clone();
        root.set_path("/.well-known/oauth-protected-resource");
        if path == root {
            vec![root]
        } else {
            vec![path, root]
        }
    };
    let protected = metadata(&client, urls).await?;
    let expected_resource = resource.to_string();
    let expected_issuer = issuer.to_owned();
    let protected = tokio::task::spawn_blocking(move || -> Result<_, Error> {
    if protected.get("signed_metadata").is_some() { return Err(failure("Signed MCP resource metadata requires unsupported verification; configure a supported resource metadata endpoint")); }
    if protected["resource"].as_str() != Some(expected_resource.as_str()) {
        return Err(failure("MCP protected-resource metadata does not match the full configured resource identity"));
    }
    let issuers = protected["authorization_servers"]
        .as_array()
        .filter(|values| !values.is_empty() && values.len() <= 16)
        .ok_or_else(|| failure("MCP resource metadata requires bounded authorization servers"))?;
    if !issuers.iter().any(|value| value.as_str() == Some(expected_issuer.as_str())) {
        return Err(failure("Configured MCP OAuth issuer is not advertised for this resource; review the definition before Connect"));
    }

        for issuer in issuers { endpoint(issuer.as_str().ok_or_else(|| failure("Invalid MCP resource authorization server"))?)?; }
        Ok(protected)
    }).await??;
    let issuer_url = endpoint(issuer)?;
    let mut urls = vec![
        well_known(&issuer_url, "oauth-authorization-server", false),
        well_known(&issuer_url, "openid-configuration", false),
    ];
    if issuer_url.path() != "/" {
        urls.push(well_known(&issuer_url, "openid-configuration", true));
    }
    let server = metadata(&client, urls).await?;
    let issuer = issuer.to_owned();
    tokio::task::spawn_blocking(move || -> Result<_, Error> {
        if server.get("signed_metadata").is_some() { return Err(failure("Signed MCP authorization metadata requires unsupported verification; configure a supported issuer")); }
        if server["issuer"].as_str() != Some(issuer.as_str()) {
            return Err(failure("MCP authorization-server metadata issuer does not exactly match the configured issuer"));
        }
        if !server["code_challenge_methods_supported"]
            .as_array()
            .is_some_and(|methods| methods.iter().any(|method| method == "S256"))
        {
            return Err(failure("MCP OAuth requires advertised PKCE S256 support"));
        }
        if !server["response_types_supported"]
            .as_array()
            .is_some_and(|types| types.iter().any(|value| value == "code"))
        {
            return Err(failure("MCP OAuth issuer does not support authorization code responses"));
        }
        for field in [
            "authorization_response_iss_parameter_supported",
            "client_id_metadata_document_supported",
        ] {
            if server.get(field).is_some_and(|value| !value.is_boolean()) {
                return Err(failure("Invalid MCP authorization-server boolean metadata"));
            }
        }
        let authorization = server["authorization_endpoint"]
            .as_str()
            .ok_or_else(|| failure("Missing MCP authorization endpoint"))?
            .to_owned();
        let token = server["token_endpoint"]
            .as_str()
            .ok_or_else(|| failure("Missing MCP token endpoint"))?
            .to_owned();
        oauth_destination(&authorization)?;
        oauth_destination(&token)?;
        let registration = server
            .get("registration_endpoint")
            .map(|value| -> Result<String, Error> {
                let value = value.as_str().ok_or_else(|| failure("Invalid MCP registration endpoint"))?;
                oauth_destination(value)?;
                Ok(value.to_owned())
            })
            .transpose()?;
        let userinfo = server
            .get("userinfo_endpoint")
            .map(|value| -> Result<String, Error> {
                let value = value.as_str().ok_or_else(|| failure("Invalid MCP userinfo endpoint"))?;
                oauth_destination(value)?;
                Ok(value.to_owned())
            })
            .transpose()?;
        let scope = if let Some(scope) = challenge.get("scope") {
            Some(scope.clone())
        } else {
            protected
                .get("scopes_supported")
                .map(|value| -> Result<String, Error> {
                    let scopes = value
                        .as_array()
                        .filter(|values| values.len() <= 128)
                        .ok_or_else(|| failure("Invalid MCP resource scopes"))?;
                    scopes
                        .iter()
                        .map(|scope| scope.as_str().ok_or_else(|| "Invalid MCP scope".into()))
                        .collect::<Result<Vec<_>, Error>>()
                        .map(|scopes| scopes.join(" "))
                })
                .transpose()?
        };
        if scope.as_ref().is_some_and(|scope| !valid_scope(scope)) {
            return Err(failure("Invalid MCP OAuth scope"));
        }
        Ok(Metadata {
            issuer: issuer.to_owned(),
            authorization,
            token,
            registration,
            userinfo,
            scope,
            iss_required: server["authorization_response_iss_parameter_supported"] == true,
            document_supported: server["client_id_metadata_document_supported"] == true,
            secret_post_supported: server["token_endpoint_auth_methods_supported"]
                .as_array()
                .is_some_and(|methods| methods.iter().any(|method| method == "client_secret_post")),
        })
    })
    .await?
}

pub(super) fn parameters(value: &str) -> Result<BTreeMap<String, String>, Error> {
    if value.len() > 16384 {
        return Err(failure("MCP authentication challenge exceeds its bound"));
    }
    let mut values = BTreeMap::new();
    let mut bearer = false;
    let mut quoted = false;
    let mut escaped = false;
    let mut start = 0;
    let mut parts = Vec::new();
    for (position, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            parts.push(&value[start..position]);
            start = position + 1;
        }
    }
    if quoted || escaped {
        return Err(failure("Unterminated MCP authentication challenge"));
    }
    parts.push(&value[start..]);
    for part in parts {
        let mut rest = part.trim();
        if let Some(space) = rest.find(char::is_whitespace) {
            if rest.find('=').is_none_or(|equal| space < equal)
                && !rest[space..].trim_start().starts_with('=')
            {
                bearer = rest[..space].eq_ignore_ascii_case("Bearer");
                rest = rest[space..].trim_start();
            }
        } else if !rest.contains('=') {
            bearer = rest.eq_ignore_ascii_case("Bearer");
            continue;
        }
        if !bearer {
            continue;
        }
        let (key, tail) = rest
            .split_once('=')
            .ok_or_else(|| failure("Invalid MCP Bearer challenge"))?;
        let key = key.trim();
        if key.is_empty()
            || !key.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(
                        byte,
                        b'!' | b'#'
                            | b'$'
                            | b'%'
                            | b'&'
                            | b'\''
                            | b'*'
                            | b'+'
                            | b'-'
                            | b'.'
                            | b'^'
                            | b'_'
                            | b'`'
                            | b'|'
                            | b'~'
                    )
            })
        {
            return Err(failure("Invalid MCP Bearer challenge parameter"));
        }
        let mut tail = tail.trim_start();
        let mut result = String::new();
        if let Some(quoted) = tail.strip_prefix('"') {
            let mut escaped = false;
            let mut end = None;
            for (position, character) in quoted.char_indices() {
                if escaped {
                    result.push(character);
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    end = Some(position);
                    break;
                } else {
                    result.push(character);
                }
            }
            let end = end.ok_or_else(|| failure("Unterminated MCP authentication challenge"))?;
            tail = &quoted[end + 1..];
            if !tail.is_empty() && !tail.trim_start().starts_with(',') {
                return Err("Malformed MCP authentication challenge separator".into());
            }
        } else {
            let end = tail.find(',').unwrap_or(tail.len());
            result.push_str(tail[..end].trim());
            tail = &tail[end..];
        }
        if result.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
            || values.insert(key.to_ascii_lowercase(), result).is_some()
        {
            return Err(failure("Invalid or duplicate MCP challenge parameter"));
        }
        if !tail.trim().is_empty() {
            return Err("Malformed MCP authentication challenge separator".into());
        }
    }
    Ok(values)
}
