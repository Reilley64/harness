mod arguments;
mod record;

pub use arguments::decode;
pub use arguments::definition;

use crate::mcp::Connections;
use crate::storage;
use crate::tool_io::Error;
use application::error_context::ResultExt;
use application::error_context::diagnostic;
use application::ports::PortFuture;
use record::FileRecord;
use record::Record;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct SnapshotRecord {
    app: String,
    repository: String,
}

#[derive(Clone)]
struct Snapshot {
    configuration: domain::McpConfiguration,
    files: SnapshotRecord,
}

struct State {
    root: PathBuf,
    registry_url: String,
    connections: Connections,
    snapshots: BTreeMap<PathBuf, Snapshot>,
}

pub fn update_mcp_configuration(
    root: &Path,
    registry_url: &str,
    connections: &Connections,
) -> impl Fn(
    PathBuf,
    domain::McpConfigurationAction,
    CancellationToken,
) -> PortFuture<Result<domain::McpConfiguration, Error>>
+ Send
+ Sync
+ use<> {
    let state = Arc::new(Mutex::new(State {
        root: root.join("mcp"),
        registry_url: registry_url.to_owned(),
        connections: connections.clone(),
        snapshots: BTreeMap::new(),
    }));

    move |workspace, action, cancellation| {
        let state = state.clone();
        Box::pin(async move {
            ensure_active(&cancellation)?;
            let workspace = tokio::fs::canonicalize(workspace).await?;
            let mut state = tokio::select! {
                biased;
                _ = cancellation.cancelled() => return Err("MCP configuration cancelled before admission".into()),
                state = state.lock() => state,
            };

            ensure_active(&cancellation)?;
            storage::private_directory(&state.root).await?;

            if matches!(action, domain::McpConfigurationAction::Reload)
                || !state.snapshots.contains_key(&workspace)
            {
                state.reload(&workspace, &cancellation).await?;
            }
            match &action {
                domain::McpConfigurationAction::Save {
                    expected_revision,
                    definition,
                } => {
                    state
                        .edit(
                            &workspace,
                            expected_revision,
                            Some(definition.clone()),
                            &definition.name,
                            &cancellation,
                        )
                        .await?;
                }
                domain::McpConfigurationAction::Remove {
                    expected_revision,
                    name,
                } => {
                    state
                        .edit(&workspace, expected_revision, None, name, &cancellation)
                        .await?;
                }
                domain::McpConfigurationAction::Connect { name, revision } => {
                    let server = state.server(&workspace, name)?;
                    if server.revision != *revision || !server.definition.enabled {
                        return Err("MCP definition changed or is disabled; review the current revision before Connect".into());
                    }
                    if matches!(
                        server.definition.transport,
                        domain::McpTransport::Http { .. }
                    ) {
                        return Err("Streamable HTTP execution/login is unavailable; definition saved only and no activation consent recorded".into());
                    }
                    ensure_active(&cancellation)?;
                    let _lock = storage::lock(&state.root.join("trust.json")).await.context("MCP trust lock is unavailable; check local permissions or a concurrent update")?;
                    let mut trust = state.trust().await?;
                    trust.insert(trust_key(&workspace, name), revision.clone());
                    if trust.len() > 1024 {
                        return Err(
                            "MCP trust storage capacity reached; remove unused local trust entries"
                                .into(),
                        );
                    }
                    let bytes = serde_json::to_vec(&trust)?;
                    ensure_active(&cancellation)?;

                    storage::replace(&state.root.join("trust.json"), &bytes).await?;
                }
                domain::McpConfigurationAction::Disconnect { name } => {
                    let server = state.server(&workspace, name)?;
                    ensure_active(&cancellation)?;
                    let _lock = storage::lock(&state.root.join("trust.json")).await.context("MCP trust lock is unavailable; check local permissions or a concurrent update")?;
                    let mut trust = state.trust().await?;
                    trust.remove(&trust_key(&workspace, name));
                    let bytes = serde_json::to_vec(&trust)?;
                    ensure_active(&cancellation)?;

                    storage::replace(&state.root.join("trust.json"), &bytes).await?;
                    state
                        .connections
                        .disconnect(&workspace, &server.revision)
                        .await;
                }
                _ => {}
            }

            let mut configuration = state.configuration(&workspace).await?;
            let connections = state.connections.clone();
            let registry_url = state.registry_url.clone();
            drop(state);
            match &action {
                domain::McpConfigurationAction::Discover { name }
                | domain::McpConfigurationAction::Connect { name, .. } => {
                    let server = configuration
                        .servers
                        .iter()
                        .find(|server| server.definition.name == *name)
                        .ok_or("MCP server not configured")?;
                    if !matches!(
                        server.definition.transport,
                        domain::McpTransport::Http { .. }
                    ) {
                        configuration.tools =
                            connections.tools(&workspace, server, cancellation).await?;
                    } else if matches!(action, domain::McpConfigurationAction::Discover { .. }) {
                        return Err("Streamable HTTP execution is unavailable; saved definition is not a connected server".into());
                    }
                }
                domain::McpConfigurationAction::Registry { query } => {
                    configuration.registry_metadata =
                        Some(registry(&registry_url, query, cancellation).await?);
                }
                _ => {}
            }

            Ok(configuration)
        })
    }
}

fn ensure_active(cancellation: &CancellationToken) -> Result<(), Error> {
    if cancellation.is_cancelled() {
        return Err("MCP configuration cancelled before admission".into());
    }

    Ok(())
}

fn trust_key(workspace: &Path, name: &str) -> String {
    digest(format!("{}\n{name}", workspace.display()).as_bytes())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

async fn read(path: &Path, private: bool) -> Result<String, Error> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok("{\"version\":1,\"servers\":[]}".into());
        }
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 262144 {
        return Err("MCP config must be a bounded regular file, not a symlink".into());
    }

    let bytes = if private {
        storage::read_private(path, 262144).await?
    } else {
        use tokio::io::AsyncReadExt;
        let mut bytes = Vec::new();
        tokio::fs::File::open(path)
            .await?
            .take(262145)
            .read_to_end(&mut bytes)
            .await?;
        if bytes.len() > 262144 {
            return Err("MCP config exceeds 256 KiB".into());
        }
        bytes
    };

    Ok(String::from_utf8(bytes)?)
}

fn parse(text: &str) -> Result<FileRecord, Error> {
    let file: FileRecord = serde_json::from_str(text)?;
    record::validate(&file)?;

    Ok(file)
}

impl State {
    async fn files(&self, workspace: &Path) -> Result<SnapshotRecord, Error> {
        Ok(SnapshotRecord {
            app: read(&self.root.join("servers.json"), true).await.context("App MCP file is inaccessible or invalid; preserve it and check owner-only permissions")?,
            repository: read(&workspace.join(".harness/mcp.json"), false).await.context("Repository MCP file is inaccessible or invalid; preserve it and check the manual configuration")?,
        })
    }

    fn snapshot(workspace: &Path, files: SnapshotRecord) -> Result<Snapshot, Error> {
        let app = parse(&files.app)
            .context("App MCP file has invalid schema or duplicate server names")?;
        let repository = parse(&files.repository)
            .context("Repository MCP file has invalid schema or duplicate server names")?;
        let revision = digest(&serde_json::to_vec(&files)?);
        let mut effective = BTreeMap::new();
        for (file, source) in [
            (app, domain::McpConfigurationSource::App),
            (repository, domain::McpConfigurationSource::Repository),
        ] {
            for record in file.servers {
                let serialized = serde_json::to_vec(&record)?;
                let material = format!(
                    "{}\n{}\n{}",
                    workspace.display(),
                    source.as_str(),
                    String::from_utf8(serialized)?
                );
                effective.insert(
                    record.name.clone(),
                    domain::McpServerStatus {
                        definition: record.try_into()?,
                        source,
                        revision: digest(material.as_bytes()),
                        status: domain::McpServerState::Untrusted,
                    },
                );
            }
        }

        Ok(Snapshot {
            configuration: domain::McpConfiguration {
                revision,
                stale: false,
                error: None,
                servers: effective.into_values().collect(),
                tools: Vec::new(),
                registry_metadata: None,
            },
            files,
        })
    }

    async fn validated(workspace: &Path, files: SnapshotRecord) -> Result<Snapshot, Error> {
        let workspace = workspace.to_owned();
        tokio::task::spawn_blocking(move || Self::snapshot(&workspace, files)).await?
    }

    async fn reload(
        &mut self,
        workspace: &Path,
        cancellation: &CancellationToken,
    ) -> Result<(), Error> {
        let path = self.root.join(format!(
            "snapshot-{}.json",
            digest(workspace.as_os_str().as_encoded_bytes())
        ));
        let candidate = match self.files(workspace).await {
            Ok(files) => Self::validated(workspace, files).await,
            Err(error) => Err(error),
        };

        match candidate {
            Ok(snapshot) => {
                let bytes = serde_json::to_vec(&snapshot.files)?;
                ensure_active(cancellation)?;

                storage::replace(&path, &bytes).await?;
                self.snapshots.insert(workspace.to_owned(), snapshot);
            }
            Err(error) => {
                if !self.snapshots.contains_key(workspace) {
                    let saved = storage::read_private(&path, 2_097_152).await.ok();
                    let files = tokio::task::spawn_blocking(move || {
                        saved
                            .and_then(|bytes| serde_json::from_slice::<SnapshotRecord>(&bytes).ok())
                    })
                    .await?;
                    let fallback = match files {
                        Some(files) => Self::validated(workspace, files).await.ok(),
                        None => None,
                    };
                    let snapshot = match fallback {
                        Some(snapshot) => snapshot,
                        None => {
                            Self::validated(
                                workspace,
                                SnapshotRecord {
                                    app: "{\"version\":1,\"servers\":[]}".into(),
                                    repository: "{\"version\":1,\"servers\":[]}".into(),
                                },
                            )
                            .await?
                        }
                    };
                    self.snapshots.insert(workspace.to_owned(), snapshot);
                }

                let snapshot = self
                    .snapshots
                    .get_mut(workspace)
                    .ok_or("Missing MCP snapshot")?;
                snapshot.configuration.stale = true;
                snapshot.configuration.error = Some(format!(
                    "{}. File preserved; using last validated snapshot. Fix the file and reload.",
                    diagnostic(error.as_ref())
                ));
            }
        }
        Ok(())
    }

    fn server(&self, workspace: &Path, name: &str) -> Result<domain::McpServerStatus, Error> {
        self.snapshots
            .get(workspace)
            .and_then(|snapshot| {
                snapshot
                    .configuration
                    .servers
                    .iter()
                    .find(|server| server.definition.name == name)
            })
            .cloned()
            .ok_or_else(|| "MCP server not configured".into())
    }

    async fn trust(&self) -> Result<BTreeMap<String, String>, Error> {
        let path = self.root.join("trust.json");
        match tokio::fs::symlink_metadata(&path).await {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(BTreeMap::new());
            }
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }

        let bytes = storage::read_private(&path, 262144).await?;
        tokio::task::spawn_blocking(move || Ok(serde_json::from_slice(&bytes)?)).await?
    }

    async fn configuration(&self, workspace: &Path) -> Result<domain::McpConfiguration, Error> {
        let mut configuration = self
            .snapshots
            .get(workspace)
            .ok_or("Missing MCP snapshot")?
            .configuration
            .clone();
        let trust = self.trust().await?;
        for server in &mut configuration.servers {
            server.status = server.definition.state(
                trust.get(&trust_key(workspace, &server.definition.name)) == Some(&server.revision),
            );
        }

        let revisions = configuration
            .servers
            .iter()
            .filter(|server| server.status == domain::McpServerState::Trusted)
            .map(|server| server.revision.clone())
            .collect::<Vec<_>>();
        self.connections.retain(workspace, &revisions).await;

        Ok(configuration)
    }

    async fn edit(
        &mut self,
        workspace: &Path,
        expected: &str,
        definition: Option<domain::McpDefinition>,
        name: &str,
        cancellation: &CancellationToken,
    ) -> Result<(), Error> {
        ensure_active(cancellation)?;
        let path = self.root.join("servers.json");
        let _lock = storage::lock(&path).await.context(
            "MCP configuration lock is unavailable; check local permissions or a concurrent update",
        )?;
        let files = self.files(workspace).await?;
        let snapshot = Self::validated(workspace, files.clone()).await?;
        if snapshot.configuration.revision != expected {
            return Err(
                "MCP configuration changed; reload and reconcile edits before saving".into(),
            );
        }

        let text = files.app.clone();
        let name = name.to_owned();
        let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Error> {
            let mut app = parse(&text)?;
            let position = app.servers.iter().position(|record| record.name == name);
            match definition {
                Some(definition) => {
                    let record = Record::from(definition);
                    if let Some(position) = position {
                        app.servers[position] = record;
                    } else {
                        app.servers.push(record);
                    }
                }
                None => {
                    if let Some(position) = position {
                        app.servers.remove(position);
                    }
                }
            }

            record::validate(&app)?;
            let bytes = serde_json::to_vec_pretty(&app)?;
            if bytes.len() > 262144 {
                return Err("MCP configuration exceeds 256 KiB".into());
            }

            Ok(bytes)
        })
        .await??;

        let current = self.files(workspace).await?;
        if current.app != files.app || current.repository != files.repository {
            return Err("Concurrent manual MCP edit detected; no file written".into());
        }
        ensure_active(cancellation)?;

        storage::replace(&path, &bytes).await?;
        self.reload(workspace, &CancellationToken::new()).await
    }
}

async fn registry(
    url: &str,
    query: &str,
    cancellation: CancellationToken,
) -> Result<String, Error> {
    if query.len() > 256 {
        return Err("Registry search exceeds 256 characters".into());
    }

    let mut url = reqwest::Url::parse(url)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || (url.scheme() != "https"
            && !(url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"))))
    {
        return Err("Registry requires HTTPS or loopback".into());
    }

    url.query_pairs_mut()
        .append_pair("search", query)
        .append_pair("limit", "10");
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let operation = async {
        let mut response = client.get(url).send().await?.error_for_status()?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 262144 {
                return Err::<String, Error>("Registry metadata exceeds 256 KiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }

        tokio::task::spawn_blocking(move || -> Result<String, Error> {
            let value: Value = serde_json::from_slice(&bytes)?;
            Ok(serde_json::to_string(&value)?)
        })
        .await?
    };

    tokio::select! { result = operation => result, _ = cancellation.cancelled() => Err("Registry search cancelled; use manual configuration or server docs".into()) }
}
