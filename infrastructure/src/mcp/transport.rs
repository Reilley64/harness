use super::encoding;
use super::http;
use super::stdio;
use crate::tool_io::Error;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) enum Transport {
    Stdio(stdio::Transport),
    Http(http::Transport),
}

impl Transport {
    pub(super) async fn start(
        auth_path: &Path,
        workspace: &Path,
        definition: &domain::McpDefinition,
        cancellation: CancellationToken,
    ) -> Result<(Self, Vec<String>), Error> {
        if cancellation.is_cancelled() {
            return Err("MCP connection cancelled before execution".into());
        }
        match &definition.transport {
            domain::McpTransport::Stdio { command, args } => {
                let (transport, secrets) = stdio::Transport::start(
                    workspace,
                    command,
                    args,
                    &definition.environment,
                    cancellation,
                )
                .await?;
                Ok((Self::Stdio(transport), secrets))
            }
            domain::McpTransport::Http {
                url,
                headers,
                oauth,
            } => {
                let (mut transport, mut secrets) =
                    http::Transport::start(auth_path, url, headers, oauth.as_ref())?;
                secrets.extend(transport.prepare(cancellation).await?);
                Ok((Self::Http(transport), secrets))
            }
            domain::McpTransport::Disabled => Err("MCP server is disabled".into()),
        }
    }

    pub(super) fn call_sent(&self) -> bool {
        match self {
            Self::Stdio(transport) => transport.call_sent(),
            Self::Http(transport) => transport.call_sent(),
        }
    }

    pub(super) fn reset_call_sent(&mut self) {
        match self {
            Self::Stdio(transport) => transport.reset_call_sent(),
            Self::Http(transport) => transport.reset_call_sent(),
        }
    }

    pub(super) fn session_identity(&self) -> Option<String> {
        match self {
            Self::Stdio(_) => None,
            Self::Http(transport) => transport.session_identity(),
        }
    }

    pub(super) fn session_lost(&self) -> bool {
        match self {
            Self::Stdio(_) => false,
            Self::Http(transport) => transport.session_lost(),
        }
    }

    pub(super) fn is_http(&self) -> bool {
        matches!(self, Self::Http(_))
    }

    pub(super) async fn prepare(
        &mut self,
        cancellation: CancellationToken,
    ) -> Result<Vec<String>, Error> {
        match self {
            Self::Stdio(_) => Ok(Vec::new()),
            Self::Http(transport) => transport.prepare(cancellation).await,
        }
    }

    pub(super) fn negotiate(&mut self, version: &str) -> Result<(), Error> {
        match self {
            Self::Stdio(_) => Ok(()),
            Self::Http(transport) => transport.negotiate(version),
        }
    }

    pub(super) async fn send(&mut self, value: serde_json::Value) -> Result<(), Error> {
        let (bytes, request, tool_call) =
            tokio::task::spawn_blocking(move || -> Result<_, Error> {
                let request = value.get("method").is_some() && value.get("id").is_some();
                let bytes = encoding::bytes(&value, 1_048_576)?;
                let tool_call = value["method"] == "tools/call";
                Ok((bytes, request, tool_call))
            })
            .await??;
        match self {
            Self::Stdio(transport) => transport.send(bytes, tool_call).await,
            Self::Http(transport) => transport.send(bytes, request, tool_call).await,
        }
    }

    pub(super) fn finish_response(&mut self) {
        if let Self::Http(transport) = self {
            transport.finish_response();
        }
    }

    pub(super) async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        match self {
            Self::Stdio(transport) => transport.receive().await,
            Self::Http(transport) => transport.receive().await,
        }
    }

    pub(super) async fn close(&mut self) -> Result<(), Error> {
        match self {
            Self::Stdio(transport) => transport.close().await,
            Self::Http(transport) => transport.close().await,
        }
    }
}
