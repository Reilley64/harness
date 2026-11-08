mod stdio;

use crate::tool_io::Error;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

type SharedClient = Arc<Mutex<Option<stdio::Client>>>;

#[derive(Clone, Default)]
pub struct Connections {
    clients: Arc<Mutex<BTreeMap<(PathBuf, String), SharedClient>>>,
}

impl Connections {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn retain(&self, workspace: &Path, revisions: &[String]) {
        let mut clients = self.clients.lock().await;
        clients.retain(|(path, revision), _| path != workspace || revisions.contains(revision));

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
            *client.lock().await = None;
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
            .map(|(tools, _)| tools)
    }

    pub(crate) async fn call(
        &self,
        workspace: &Path,
        server: &domain::McpServerStatus,
        arguments: domain::McpCallArguments,
        cancellation: CancellationToken,
    ) -> Result<domain::ToolResult, Error> {
        let (_, result) = self
            .request(workspace, server, Some(arguments), cancellation)
            .await?;
        result.ok_or_else(|| "Missing MCP tool result".into())
    }

    async fn request(
        &self,
        workspace: &Path,
        server: &domain::McpServerStatus,
        arguments: Option<domain::McpCallArguments>,
        cancellation: CancellationToken,
    ) -> Result<(Vec<domain::McpTool>, Option<domain::ToolResult>), Error> {
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
            guard = client.lock() => guard,
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
                stdio::Client::start(workspace, &server.definition, cancellation.clone()).await?
            }
        };

        let result = client.perform(arguments, cancellation).await;
        if result.as_ref().is_ok_and(|(_, result)| {
            result
                .as_ref()
                .is_none_or(|result| result.status != domain::ToolStatus::Unknown)
        }) {
            *guard = Some(client);
        }

        result
    }
}
