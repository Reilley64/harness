use super::super::failure;
use crate::tool_io::Error;
use application::error_context::ResultExt;
use axum::Router;
use axum::extract::RawQuery;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::routing::get;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub(super) struct Callback {
    pub receiver: mpsc::Receiver<Result<String, Error>>,
    task: JoinHandle<()>,
}

impl Drop for Callback {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(super) async fn bind(redirect: Option<&str>) -> Result<(TcpListener, String), Error> {
    let port = if let Some(redirect) = redirect {
        validate_redirect(redirect)?
            .port()
            .ok_or_else(|| failure("MCP OAuth redirect requires an explicit loopback port"))?
    } else {
        0
    };
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .context(
            "MCP OAuth callback port is unavailable; configure a free registered loopback redirect",
        )?;
    let actual_redirect = format!(
        "http://127.0.0.1:{}/mcp/callback",
        listener.local_addr()?.port()
    );
    if redirect.is_some_and(|redirect| redirect != actual_redirect) {
        return Err(failure(
            "MCP OAuth registered redirect does not match the bound callback",
        ));
    }
    Ok((listener, actual_redirect))
}

pub(super) async fn start(
    listener: TcpListener,
    state: &str,
    issuer: &str,
    iss_required: bool,
    client_id: &str,
    resource: &str,
) -> Result<Callback, Error> {
    let authority = format!("127.0.0.1:{}", listener.local_addr()?.port());
    let state = state.to_owned();
    let issuer = issuer.to_owned();
    let client_id = client_id.to_owned();
    let resource = resource.to_owned();
    let claimed = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel(1);
    let router = Router::new().route("/mcp/callback", get(move |RawQuery(query): RawQuery, headers: HeaderMap| {
        let state=state.clone(); let issuer=issuer.clone(); let client_id=client_id.clone(); let resource=resource.clone(); let claimed=claimed.clone(); let sender=sender.clone(); let authority=authority.clone();
        async move {
            if headers.get_all("host").iter().count() != 1 || headers.get("host").and_then(|value| value.to_str().ok()) != Some(authority.as_str()) { return (StatusCode::BAD_REQUEST, "Invalid callback destination. Return to Harness."); }
            let values = match parse(query.as_deref().unwrap_or("")) { Ok(values) => values, Err(_) => return (StatusCode::BAD_REQUEST, "Invalid authorization callback. Return to Harness.") };
            if !values.get("state").is_some_and(|value| constant_time_eq::constant_time_eq(value.as_bytes(), state.as_bytes())) { return (StatusCode::BAD_REQUEST, "Invalid authorization state. Return to Harness."); }
            if values.get("iss").is_some_and(|value| value != &issuer) || (iss_required && !values.contains_key("iss")) || values.get("client_id").is_some_and(|value| value != &client_id) || values.get("resource").is_some_and(|value| value != &resource) { return (StatusCode::BAD_REQUEST, "Authorization identity mismatch. Return to Harness."); }
            let result = match (values.get("code"), values.get("error")) {
                (Some(code), None) if !code.is_empty() && code.len() <= 8192 && !code.chars().any(char::is_control) => Ok(code.clone()),
                (None, Some(_)) => Err(failure("MCP authorization was denied; pending work and existing credentials are preserved")),
                _ => return (StatusCode::BAD_REQUEST, "Invalid authorization response. Return to Harness."),
            };
            if claimed.swap(true, Ordering::AcqRel) || sender.try_send(result).is_err() { return (StatusCode::CONFLICT, "Authorization attempt already completed. Return to Harness."); }
            (StatusCode::OK, "Authorization received. Return to Harness to check the result.")
        }
    }));
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok(Callback { receiver, task })
}

pub(super) fn validate_redirect(value: &str) -> Result<reqwest::Url, Error> {
    let url = reqwest::Url::parse(value).context("Invalid MCP OAuth redirect")?;
    if value.len() > 1024
        || url.scheme() != "http"
        || url.host_str() != Some("127.0.0.1")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/mcp/callback"
        || url.port().is_none_or(|port| port == 0)
    {
        return Err(failure(
            "MCP OAuth redirect must be http://127.0.0.1:<registered-port>/mcp/callback",
        ));
    }
    Ok(url)
}

fn parse(value: &str) -> Result<BTreeMap<String, String>, Error> {
    if value.len() > 16384 {
        return Err(failure("MCP OAuth callback exceeds its bound"));
    }
    let bytes = value.as_bytes();
    for (position, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && (position + 2 >= bytes.len()
                || !bytes[position + 1].is_ascii_hexdigit()
                || !bytes[position + 2].is_ascii_hexdigit())
        {
            return Err(failure("Malformed MCP callback encoding"));
        }
    }
    let mut values = BTreeMap::new();
    for (key, value) in reqwest::Url::parse(&format!("http://127.0.0.1/?{value}"))?.query_pairs() {
        if key.contains('\u{fffd}')
            || value.contains('\u{fffd}')
            || values
                .insert(key.into_owned(), value.into_owned())
                .is_some()
        {
            return Err(failure("Duplicate or invalid MCP callback parameter"));
        }
    }
    Ok(values)
}
