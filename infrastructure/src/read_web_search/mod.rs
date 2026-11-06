use crate::conversations;
use crate::tool_io;

mod arguments;
pub(crate) fn decode(json: &str) -> Result<domain::ToolInvocation, tool_io::Error> {
    arguments::decode(json)
}
pub(crate) fn definition() -> serde_json::Value {
    arguments::definition()
}
use crate::tool_io::Error;
use crate::tool_io::Future;
use application::error_context::ResultExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub fn read_web_search(
    _root: &Path,
) -> impl Fn(PathBuf, domain::ReadWebSearchArguments, CancellationToken) -> Future + Send + Sync + use<>
{
    move |_, arguments, cancellation| Box::pin(search(arguments, cancellation))
}
pub(crate) async fn search(
    arguments: domain::ReadWebSearchArguments,
    cancellation: CancellationToken,
) -> Result<domain::ToolResult, Error> {
    if arguments.query.trim().is_empty() || arguments.query.len() > 4096 {
        return Err("Search query must contain 1–4096 bytes".into());
    }
    let operation = async {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
            .build()?;
        let mut response = client
            .get("https://html.duckduckgo.com/html/")
            .query(&[("q", &arguments.query)])
            .send()
            .await
            .context("Could not contact DuckDuckGo")?;
        if !response.status().is_success() || response.status().as_u16() == 202 {
            return Err(format!(
                "DuckDuckGo search was blocked or throttled (HTTP {})",
                response.status().as_u16()
            )
            .into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
                return Err("DuckDuckGo response exceeds 2 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        let html = String::from_utf8(bytes).context("Invalid DuckDuckGo response encoding")?;
        tokio::task::spawn_blocking(move || {
            parse(
                &html,
                &arguments.query,
                arguments.limit.unwrap_or(5).clamp(1, 10),
            )
        })
        .await?
    };
    tokio::select! {_=cancellation.cancelled()=>Ok(domain::ToolResult::cancelled()),result=tokio::time::timeout(Duration::from_secs(15),operation)=>result.context("DuckDuckGo search timed out")?}
}
fn parse(html: &str, query: &str, limit: usize) -> Result<domain::ToolResult, Error> {
    let lower = html.to_lowercase();
    if [
        "anomaly.js",
        "challenge-form",
        "anomaly-modal",
        "bots use duckduckgo",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return Err("DuckDuckGo returned a challenge; no results were accepted".into());
    }
    let document = scraper::Html::parse_document(html);
    let results = scraper::Selector::parse(".result")
        .map_err(|e| Error::from(format!("Invalid result selector: {e}")))?;
    let anchor = scraper::Selector::parse(".result__a")
        .map_err(|e| Error::from(format!("Invalid anchor selector: {e}")))?;
    let snippet = scraper::Selector::parse(".result__snippet")
        .map_err(|e| Error::from(format!("Invalid snippet selector: {e}")))?;
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for result in document.select(&results) {
        if result.value().classes().any(|class| class == "result--ad") {
            continue;
        }
        let Some(a) = result.select(&anchor).next() else {
            continue;
        };
        let title = a.text().collect::<String>();
        let Some(href) = a.value().attr("href") else {
            continue;
        };
        let absolute = if href.starts_with("//") {
            format!("https:{href}")
        } else {
            href.to_owned()
        };
        let Ok(mut url) = reqwest::Url::parse(&absolute) else {
            continue;
        };
        let decoded = url
            .query_pairs()
            .find(|(key, _)| key == "uddg")
            .map(|(_, value)| value.into_owned());
        if let Some(decoded) = decoded {
            let Ok(target) = reqwest::Url::parse(&decoded) else {
                continue;
            };
            url = target;
        }
        if url
            .host_str()
            .is_some_and(|host| host == "duckduckgo.com" || host.ends_with(".duckduckgo.com"))
            && url.path() == "/y.js"
        {
            continue;
        }
        if !matches!(url.scheme(), "http" | "https")
            || title.trim().is_empty()
            || url.as_str().len() > 8192
            || !seen.insert(url.to_string())
        {
            continue;
        }
        let content = result
            .select(&snippet)
            .next()
            .map(|s| s.text().collect::<String>())
            .unwrap_or_default();
        output.push(
            serde_json::json!({"title":title.trim(),"url":url.as_str(),"snippet":content.trim()}),
        );
        if output.len() >= limit {
            break;
        }
    }
    if output.is_empty() && !lower.contains("no-results") && !lower.contains("no results found") {
        return Err(
            "DuckDuckGo response layout was not recognized; this is not an empty search".into(),
        );
    }
    Ok(domain::ToolResult::success(serde_json::json!({"provider":"duckduckgo","query":query,"retrievedAt":conversations::now()?,"results":output,"limit":limit}).to_string()))
}
