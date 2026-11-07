use application::ports::PortFuture;
use sha2::Digest;
use sha2::Sha256;
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;
type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Arguments {
    work_id: String,
    intent: String,
    path: String,
    revision: Option<u64>,
}
pub(crate) fn decode(value: &str) -> Result<domain::ToolInvocation, Error> {
    let a: Arguments = serde_json::from_str(value)?;
    if a.path.is_empty() || a.path.len() > 4096 {
        return Err("Proposal path is invalid".into());
    }
    Ok(domain::ToolInvocation::PublishWork(
        domain::PublishWorkArguments {
            work_id: a.work_id,
            intent: a.intent,
            path: a.path,
            revision: a.revision,
        },
    ))
}
pub(crate) fn definition() -> serde_json::Value {
    serde_json::json!({"type":"function","name":"publish_work","description":"Capture a reviewable proposal JSON file after publishing through this repository's supported file/Bash tracker conventions. This creates no human authority. Missing tracker access must be reported, not fabricated. Fields: root (source URI/path), source_revision, spec, scope, criteria, dependencies (graph text), wiki and context (pointer or null), artifacts (pointer array). Use canonical UUID workId and publication intent. For amendments supply current revision and a new intent; identical intent/payload is idempotent.","parameters":{"type":"object","properties":{"workId":{"type":"string"},"intent":{"type":"string"},"path":{"type":"string"},"revision":{"type":["integer","null"],"minimum":1}},"required":["workId","intent","path"],"additionalProperties":false}})
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    root: String,
    source_revision: String,
    spec: String,
    scope: String,
    criteria: String,
    dependencies: String,
    wiki: Option<String>,
    context: Option<String>,
    artifacts: Vec<String>,
}
pub fn read_work_proposal() -> impl Fn(
    std::path::PathBuf,
    String,
    CancellationToken,
) -> PortFuture<Result<domain::WorkProposal, Error>>
+ Send
+ Sync {
    move |workspace, path, cancellation| {
        Box::pin(async move {
            let read = async move {
                let mut options = tokio::fs::OpenOptions::new();
                options.read(true);
                #[cfg(unix)]
                options.custom_flags(libc::O_NONBLOCK);
                let file = options.open(workspace.join(path)).await?;
                if !file.metadata().await?.is_file() {
                    return Err("Proposal must be a regular UTF-8 file".into());
                }
                let mut bytes = Vec::new();
                file.take(262_145).read_to_end(&mut bytes).await?;
                if bytes.len() > 262_144 {
                    return Err("Proposal exceeds 256 KiB".into());
                }
                tokio::task::spawn_blocking(move || {
                    let document = String::from_utf8(bytes)?;
                    let r: Response = serde_json::from_str(&document)?;
                    let version = format!("{:x}", Sha256::digest(document.as_bytes()));
                    let p = domain::WorkProposal {
                        root: r.root,
                        source_revision: r.source_revision,
                        spec: r.spec,
                        scope: r.scope,
                        criteria: r.criteria,
                        dependencies: r.dependencies,
                        wiki: r.wiki,
                        context: r.context,
                        artifacts: r.artifacts,
                        document,
                        version,
                    };
                    p.validate()?;
                    Ok(p)
                })
                .await?
            };
            tokio::select! {
                biased;
                _ = cancellation.cancelled() => Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Proposal read cancelled").into()),
                result = read => result,
            }
        })
    }
}
