# MCP configuration and authorization over ACP

## MCP configuration extension

Initialization advertises `_meta["harness/mcp"]: true`.
`_harness/mcp` accepts an explicit `workspace` and a tagged `action`:
`read`, `reload`, `save` (`expected_revision`, `definition`), `remove`
(`expected_revision`, `name`), `registry` (`query`), `discover` (`name`),
`connect` (`name`, exact definition `revision`) or `disconnect` (`name`).
Definitions use the documented [MCP configuration schema](../mcp-connections.md).
Responses contain `revision`, `stale`, `error`, `servers` (nonsecret definition,
source, revision, eligibility status), `tools` (names/descriptions/input schemas)
and optional `registry_metadata`.

### MCP OAuth challenge correlation

OAuth Connect also requires a top-level `authentication_id`: a fresh opaque
string of 1–128 UTF-8 bytes, distinct from the JSON-RPC request ID. It is optional
for other operations and for Connect without OAuth. A missing or unusable ID
cannot supply the authorization presenter required by OAuth Connect.

After reviewing and approving the exact nonsecret definition, a trusted client
can send:

```json
{
  "jsonrpc": "2.0",
  "id": "mcp-connect-1",
  "method": "_harness/mcp",
  "params": {
    "workspace": "/absolute/workspace",
    "action": "connect",
    "name": "oauth-tools",
    "revision": "<exact server revision>",
    "authentication_id": "<fresh opaque authorization ID>"
  }
}
```

While that request is pending, the backend can send this Harness notification:

```json
{
  "jsonrpc": "2.0",
  "method": "_harness/mcp/authorization",
  "params": {
    "authentication_id": "<fresh opaque authorization ID>",
    "workspace": "/absolute/workspace",
    "revision": "<exact server revision>",
    "authorization_url": "https://accounts.example.invalid/authorize"
  }
}
```

The challenge echoes the request's authentication ID, workspace and definition
revision. Register the pending presentation before sending Connect; accept a link
only when all three match the active operation. Invalidate its presentation on
cancellation, replacement, dismissal or completion, and ignore stale or unrelated
challenges. Treat the URL as protected native authorization data, not chat,
model-tool output, durable conversation history or a diagnostic log. The current
TUI hides it until explicit reveal/open; see [frontend](../frontend.md#mcp-configuration).
A challenge is not a successful login or connection result: await the original
`_harness/mcp` response and reconcile failures through read/reload rather than
resending effects blindly.

### MCP cancellation and supported transports

Native command and agent configuration invocation share the public application
use case/backend. The model-facing decoder rejects Connect/Disconnect; only the
trusted native/local ACP client presents those interactions. ACP is a trusted
same-user interface, not an authenticated permission boundary against arbitrary
local clients. It never accepts actual environment/header secret values or
provider tokens. MCP errors use `-32006` with safe contexts; invalid requests use
`-32602`. Connection-owned tasks preserve request responsiveness. SDK request
cancellation signals that operation, but the backend still awaits its completion:
admitted configuration and credential commits are not abandoned or reported as
rolled back. Clients should keep the response wait while cancelling; the current
TUI uses `stopWaitingOnCancel: false` and includes pending MCP requests in its
bounded 60-second shutdown drain. A lost response requires read/reload to reconcile
completed state, not an assumed rollback or automatic replay. Closing a panel or
cancelling one consumer is not a global Disconnect of shared tools.

Native startup reads configuration for its bound workspace without launching
servers or logging in. Standard ACP client-supplied `mcpServers` remain unused;
Harness owns effective configuration and local activation rather than trusting a
session parameter as executable consent. Both stdio and Streamable HTTP tools use
this shared backend. See [HTTP and OAuth](../mcp-http-auth.md) for the supported
protocol/client profiles, private token lifecycle, bounded transport and recovery
limits; model-provider authentication remains separate.
