mod client;
mod encoding;
mod http;
mod stdio;
mod transport;

use crate::storage;
use crate::tool_io::Error;
use application::ports::PresentMcpAuthorization;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub(crate) fn validate_http(
    url: &str,
    headers: &BTreeMap<String, String>,
    oauth: Option<&domain::McpOauth>,
) -> Result<(), Error> {
    http::endpoint(url)?;
    let mut names = BTreeSet::new();
    if headers
        .keys()
        .any(|name| !names.insert(name.to_ascii_lowercase()))
    {
        return Err("Duplicate case-insensitive MCP HTTP header".into());
    }
    if headers.keys().any(|name| {
        http::reserved_header(name)
            || (oauth.is_some() && name.eq_ignore_ascii_case("authorization"))
    }) {
        return Err("MCP HTTP header is reserved or conflicts with OAuth".into());
    }
    if let Some(oauth) = oauth {
        http::validate_oauth(oauth)?;
    }
    Ok(())
}

#[derive(Default)]
struct Slot {
    client: Mutex<Option<client::Client>>,
    authentication: CancellationToken,
}

type SharedClient = Arc<Slot>;

pub(crate) struct ConnectPermit {
    authorization: Option<http::Authorization>,
    cancellation: CancellationToken,
}

#[derive(Clone)]
pub struct Connections {
    root: PathBuf,
    clients: Arc<Mutex<BTreeMap<(PathBuf, String), SharedClient>>>,
}

impl Connections {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
            clients: Arc::default(),
        }
    }

    pub(crate) async fn admit_connect(
        &self,
        workspace: &Path,
        server: &domain::McpServerStatus,
        cancellation: CancellationToken,
    ) -> Result<ConnectPermit, Error> {
        if cancellation.is_cancelled() {
            return Err("MCP Connect cancelled before authorization admission".into());
        }
        let slot = self
            .clients
            .lock()
            .await
            .get(&(workspace.to_owned(), server.revision.clone()))
            .cloned()
            .ok_or("MCP Connect was superseded or disconnected")?;
        let authorization = if matches!(
            server.definition.transport,
            domain::McpTransport::Http { oauth: Some(_), .. }
        ) {
            Some(http::admit(&self.auth_path(&server.revision).await?, cancellation).await?)
        } else {
            None
        };
        Ok(ConnectPermit {
            authorization,
            cancellation: slot.authentication.child_token(),
        })
    }

    pub(crate) async fn authenticate(
        &self,
        permit: ConnectPermit,
        server: &domain::McpServerStatus,
        present: Option<&PresentMcpAuthorization>,
        cancellation: CancellationToken,
    ) -> Result<(), Error> {
        if let domain::McpTransport::Http {
            url,
            oauth: Some(config),
            ..
        } = &server.definition.transport
        {
            let present = present.ok_or(
                "MCP OAuth requires explicit native Connect with authorization presentation",
            )?;
            let combined = permit.cancellation;
            if cancellation.is_cancelled() {
                combined.cancel();
            }
            let trigger = combined.clone();
            let watcher = tokio::spawn(async move {
                cancellation.cancelled().await;
                trigger.cancel();
            });
            let attempt = permit
                .authorization
                .ok_or("Missing MCP authorization admission")?;
            let result = http::authenticate(attempt, url, config, present, combined).await;
            watcher.abort();
            result?;
        }
        Ok(())
    }

    async fn auth_path(&self, revision: &str) -> Result<PathBuf, Error> {
        storage::private_directory(&self.root).await?;
        storage::private_directory(&self.root.join("mcp")).await?;
        let root = self.root.join("mcp/auth");
        storage::private_directory(&root).await?;
        if revision.len() != 64 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("Invalid MCP credential identity".into());
        }
        Ok(root.join(format!("{revision}.json")))
    }

    pub(crate) async fn revoke(&self, server: &domain::McpServerStatus) -> Result<(), Error> {
        if matches!(
            server.definition.transport,
            domain::McpTransport::Http { oauth: Some(_), .. }
        ) {
            http::disconnect(&self.auth_path(&server.revision).await?).await?;
        }
        Ok(())
    }

    pub(crate) async fn retain(&self, workspace: &Path, revisions: &[String]) {
        let mut clients = self.clients.lock().await;
        clients.retain(|(path, revision), slot| {
            let retained = path != workspace || revisions.contains(revision);
            if !retained {
                slot.authentication.cancel();
            }
            retained
        });

        for revision in revisions {
            clients
                .entry((workspace.to_owned(), revision.clone()))
                .or_default();
        }
    }

    pub(crate) async fn disconnect(&self, workspace: &Path, revision: &str) {
        let client = self
            .clients
            .lock()
            .await
            .remove(&(workspace.to_owned(), revision.to_owned()));
        if let Some(client) = client {
            client.authentication.cancel();
        }
    }

    pub(crate) async fn tools(
        &self,
        workspace: &Path,
        server: &domain::McpServerStatus,
        cancellation: CancellationToken,
    ) -> Result<Vec<domain::McpTool>, Error> {
        self.request(workspace, server, None, cancellation)
            .await
            .map(|(tools, _, _)| tools)
    }

    pub(crate) async fn call(
        &self,
        workspace: &Path,
        server: &domain::McpServerStatus,
        arguments: domain::McpCallArguments,
        cancellation: CancellationToken,
    ) -> Result<(domain::ToolResult, Option<Error>), Error> {
        let (_, result, cause) = self
            .request(workspace, server, Some(arguments), cancellation)
            .await?;
        Ok((result.ok_or("Missing MCP tool result")?, cause))
    }

    async fn request(
        &self,
        workspace: &Path,
        server: &domain::McpServerStatus,
        arguments: Option<domain::McpCallArguments>,
        cancellation: CancellationToken,
    ) -> Result<
        (
            Vec<domain::McpTool>,
            Option<domain::ToolResult>,
            Option<Error>,
        ),
        Error,
    > {
        if server.status != domain::McpServerState::Trusted {
            return Err(
                "MCP server is not enabled and trusted; use /mcp to review and Connect".into(),
            );
        }

        let key = (workspace.to_owned(), server.revision.clone());
        let client = self
            .clients
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or("MCP trust was revoked or configuration changed")?;
        let mut guard = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err("MCP operation cancelled before execution".into()),
            guard = client.client.lock() => guard,
        };
        if !self
            .clients
            .lock()
            .await
            .get(&key)
            .is_some_and(|current| Arc::ptr_eq(current, &client))
        {
            return Err("MCP trust was revoked while waiting; no call sent".into());
        }
        if cancellation.is_cancelled() {
            return Err("MCP operation cancelled before execution".into());
        }

        let mut client = match guard.take() {
            Some(client) => client,
            None => {
                let auth_path = self.auth_path(&server.revision).await?;
                let (transport, redactions) = transport::Transport::start(
                    &auth_path,
                    workspace,
                    &server.definition,
                    cancellation.clone(),
                )
                .await?;
                client::Client::start(transport, redactions, cancellation.clone()).await?
            }
        };

        let result = client.perform(arguments, cancellation).await;
        let cause = client.take_cause();
        if client.is_http()
            || result.as_ref().is_ok_and(|(_, result)| {
                result
                    .as_ref()
                    .is_none_or(|result| result.status != domain::ToolStatus::Unknown)
            })
        {
            *guard = Some(client);
        }

        let (tools, result) = result?;
        Ok((tools, result, cause))
    }
}
