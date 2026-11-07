# ACP presentation

The terminal frontend is implemented in `presentation/tui` with React and Ink,
using Bun as its package manager and runtime. It follows
[atomic design](frontend.md), with rendering separated from hooks and ACP transport.
It spawns the Rust `harness` executable, whose `presentation/acp` adapter uses
the official Rust ACP SDK. The frontend uses the official TypeScript SDK.
Both select stable ACP v1; no unstable provider/auth extensions are enabled.

## Responsibility split

- Ink owns terminal layout, prompt editing, command parsing, picker state, and
  keyboard navigation.
- ACP presents application behavior to external clients over newline-delimited
  JSON-RPC on stdio. It maps domain providers into presentation DTOs.
- Application owns catalog reading, provider connection creation, conversation
  creation/loading, and message creation through closure-backed ports.
- Infrastructure owns configuration, HTTP DTOs, cache wrappers, and adapter
  composition.

The root executable builds infrastructure dependencies once and derives the
application use-case bundles into presentation-owned `presentation_acp::Dependencies`
using its `fundle` builder. `presentation_acp::run` accepts that lean bundle (or
`Into<Dependencies>`); ACP has no infrastructure dependency. Handlers call
application operations rather than infrastructure adapters directly.

## Initialization

Send the standard ACP `initialize` request with `protocolVersion: 1`.
The response identifies the agent as `harness`, enables `agentCapabilities.loadSession`,
advertises the agent-handled
`openai` authentication method (Continue with ChatGPT), and includes:

```json
{
  "_meta": {
    "harness/providerCatalog": true,
    "harness/modelCatalog": true,
    "harness/modelSelection": true,
    "harness/tools": true,
    "harness/steering": true
  }
}
```

The frontend checks both the negotiated protocol version and the extension marker
before requesting providers, and checks the model marker before requesting models.

## MCP configuration extension

Initialization advertises `_meta["harness/mcp"]: true`.
`_harness/mcp` accepts an explicit `workspace` and a tagged `action`:
`read`, `reload`, `save` (`expected_revision`, `definition`), `remove`
(`expected_revision`, `name`), `registry` (`query`), `discover` (`name`),
`connect` (`name`, exact definition `revision`) or `disconnect` (`name`).
Definitions use the documented [MCP configuration schema](mcp-connections.md).
Responses contain `revision`, `stale`, `error`, `servers` (nonsecret definition,
source, revision, eligibility status), `tools` (names/descriptions/input schemas)
and optional `registry_metadata`.

Native command and agent configuration invocation share the public application
use case/backend. The model-facing decoder rejects Connect/Disconnect; only the
trusted native/local ACP client presents those interactions. ACP is a trusted
same-user interface, not an authenticated permission boundary against arbitrary
local clients. It never accepts actual environment/header secret values or
provider tokens. MCP errors use `-32006` with safe contexts; invalid requests use
`-32602`. Connection-owned tasks preserve request responsiveness; cancellation
signals the shared operation without claiming to undo committed saves/effects.
Native startup reads configuration for its bound workspace without launching
servers. Standard ACP client-supplied `mcpServers` remain unused; Harness owns
effective configuration and local activation rather than trusting a session
parameter as executable consent. MCP HTTP execution remains unavailable.

## Provider catalog extension

Request:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "_harness/providers",
  "params": {}
}
```

Successful result:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "providers": [{ "id": "openai", "name": "ChatGPT" }]
  }
}
```

The method reads through the application port, including its existing memory cache
and disk fallback. Errors return JSON-RPC code `-32001`, message
`Could not load providers`, and a static safe diagnostic in `data`. Provider
URLs, paths, and raw error bodies are not published. Unknown methods return
`-32601`; unexpected catalog parameters return `-32602`.

This is a Harness extension, not the unstable ACP `providers/list` method.
The catalog describes available providers; it does not claim they support a
working login flow.

## Model catalog extension

`_harness/models` accepts an empty object and calls `read_models`, with the same
memory-cache and file-fallback composition used by the provider catalog.
Unexpected parameters return `-32602`. Catalog errors return `-32003` with
message `Could not load models` and a static safe diagnostic in `data`.

A result uses camelCase presentation fields, unlike the HTTP catalog's snake_case:

```json
{
  "models": [
    {
      "id": "example-model",
      "name": "Example Model",
      "providerId": "openai",
      "efforts": ["low", "medium", "high", "extra_high"],
      "pricing": { "input": 1.25, "cachedInput": 0.25, "output": 10 }
    }
  ]
}
```

These example rates are illustrative USD per million tokens. A null pricing
object or individual null value means unavailable, while numeric zero means free.
An empty efforts array means no effort control is available. Domain types remain
Serde-free; infrastructure and ACP have their own DTOs.

Requests run as connection-owned tasks so cancellation and other ACP requests
stay responsive while loading. Closing the Ink picker aborts the request and
suppresses stale results. Selecting a model updates frontend state; the next
session creation/prompt validates and persists the selection.

## Saved model selection

`_harness/model-selection/read` accepts `{}` and returns
`{"selection": null}` when no preference exists, otherwise
`{"selection": {"model": <model summary>, "effort": "extra_high"}}`.
The model summary has the same shape as a catalog entry. Reading requires no
catalog, credentials, session creation, or inference.

`_harness/model-selection/update` accepts
`{"providerId": "openai", "modelId": "gpt-6.1-sol", "effort": "extra_high"}`.
Effort may be null. It validates the catalog model and supported effort, atomically
saves the preference, and returns the same selection response shape. Unknown
fields or malformed effort values use `-32602`; validation/storage failures use
`-32005` with a safe diagnostic. Preferences live under the shared app-data root.
Explicit session load does not read or overwrite this default; actual sends
remember the validated selection used for that message.

## Provider connection creation

Typing `/auth` opens the picker and loads the catalog asynchronously. Selecting
ChatGPT calls standard ACP `authenticate` with `methodId: "openai"`. The adapter
calls `create_provider_connection::call`; the use case selects a provider-bound
`CreateProviderConnection` through `CreateProviderConnectionFactory`, then invokes
it. Infrastructure owns provider selection, OAuth, and credential persistence.

Ink adds a fresh correlation identifier to each authentication request:

```json
{
  "methodId": "openai",
  "_meta": { "harness/authenticationId": "<opaque-attempt-id>" }
}
```

Before authentication completes, the agent sends a Harness notification:

```json
{
  "jsonrpc": "2.0",
  "method": "_harness/authentication",
  "params": {
    "authenticationId": "<opaque-attempt-id>",
    "providerId": "openai",
    "authorizationUrl": "https://auth.openai.com/api/accounts/authorize?..."
  }
}
```

Rust starts the loopback callback listener before presenting the challenge and
opens the browser by default. Ink displays progress and can reopen the link on
explicit user input. The full sign-in URL is hidden by default; `l` explicitly
reveals/hides it when manual sign-in is needed. The correlation identifier is optional for other ACP
clients; their notifications have a null `authenticationId`.

A successful `authenticate` response includes only safe connection information:

```json
{
  "_meta": {
    "harness/providerConnection": {
      "id": "<connection-id>",
      "providerId": "openai"
    }
  }
}
```

No access, refresh, or ID token, OIDC subject, or account email/display label
crosses ACP. Connection metadata contains only the opaque local connection ID
and provider ID. Authorization URLs omit ID-token
hints. Failure returns `-32002` with message `Could not create provider connection`
and diagnostic text in `data`. Unsupported authentication methods return
`-32602`. Ink checks advertised methods before starting a flow.

Escape aborts the client request using ACP `$/cancel_request`. A cancelled
request returns `-32800`; dropping the operation closes the callback listener
and releases its credential lock. Connection-owned tasks keep the ACP dispatch
loop free to process cancellation and other requests while sign-in is pending.

Client-hosted filesystem/terminal methods and permission requests remain
unimplemented; enabled tools execute in the agent. See
[provider connections](provider-connections.md) for OAuth and credential storage.

## Conversations

Standard ACP v1 methods present the conversation use cases:

- `session/new` calls `create_conversation`. Ink waits until the first message
  before requesting it. Its `_meta["harness/modelSelection"]` is required:
  `{ "providerId": "openai", "modelId": "<catalog-id>", "effort": "medium" }`.
  Effort may be null. The backend resolves the saved provider connection; no
  credentials or tokens are supplied by the frontend.
- `session/load` calls `read_conversation` with an explicit canonical UUID.
  It replays ordered user/agent chunks and tool calls at their persisted UTF-8
  text offsets, retaining stable message/tool IDs and Harness status metadata.
  Tool replay includes an initial `tool_call` and current `tool_call_update`.
  It returns authoritative conversation metadata, including partial statuses,
  without inference or credentials/catalog access.
- `session/prompt` accepts baseline text and `resource_link` blocks and calls
  `create_message`. Resource links become JSON-encoded user-content references
  marked as unread; they never trigger fetching or imply access to their contents.
  Optional image, audio and embedded-resource blocks remain unsupported. Mixed
  text retains its whitespace and the combined message has a 1 MiB UTF-8 limit.
  Optional model
  selection metadata changes the next reply's model/effort within the same
  provider. The user message and in-progress assistant are stored first.
- `session/update` carries `agent_message_chunk` text with a stable `messageId`
  during inference. Credentials and opaque reasoning events are never forwarded.
- `session/cancel` signals the active reply. The prompt finishes only after the
  cancelled partial assistant is saved, then returns `stopReason: "cancelled"`.
  JSON-RPC request cancellation also signals rather than dropping message creation.

New/load/prompt responses include `_meta["harness/conversation"]`: ID, provider
connection ID, model/effort, ordered messages with stable IDs/roles/content/status
and per-reply model/effort, timestamps, and revision. This is a safe presentation
snapshot, never the credential record. Incomplete and refused replies map to
`max_tokens` and `refusal`; normal completion maps to `end_turn`.

Sessions must be created or loaded in this ACP process before prompting. Prompt
overlap is rejected locally and by the cross-process conversation lock.
Conversation errors use `-32004` with message `Conversation request failed` and
diagnostic text in `data`; malformed selection/content uses `-32602`. Generation
failure still saves partial text as failed. The frontend reloads the authoritative
snapshot after such errors rather than claiming an optimistic message was saved.

See [conversations](conversations.md) for domain rules and durability limits.

## Lifecycle

The frontend uses the terminal alternate screen and restores it on exit.
Dismissing a pending picker aborts the local wait and requests ACP cancellation;
the Rust reader may still finish its in-flight catalog fetch. Late results do
not reopen a dismissed picker. Catalog retry remains subject to the existing TTL.

The frontend drains and discards raw agent stderr. Only explicit safe error
contexts cross the diagnostic boundary; arbitrary underlying error text is not
published. On exit it requests cancellation of active replies and waits for
their persistence, allowing up to 60 seconds for an in-flight credential refresh.
Then it closes the ACP connection and agent stdin, waits for the agent, and
escalates to process termination if necessary. The Rust agent exits
when its input closes. Stdout is exclusively protocol traffic.

## Development

```sh
mise install
mise run
mise run typecheck
mise run build
```

The default Mise task builds the managed agent through `build:agent` and uses
`target/debug/harness` (`harness.exe` on Windows).
The UI needs an interactive terminal; another ACP client can launch the Rust
binary directly without a TTY.

## Token usage and workspace

Conversation metadata includes the canonical bound `workspace` and a `usage`
summary with nullable `inputTokens`, `outputTokens`, `contextTokens`, `modelId`,
and a boolean `partial`. Input/output are cumulative reported session tokens;
context is the latest request's input plus output, not the cumulative total.
Missing reports remain unknown; partial totals are lower bounds.

The `harness/usage` capability advertises `_harness/usage` notifications:
`{sessionId, usage}`. Reports are emitted after durable generation accounting;
retained model steps are checkpointed before their updates, including steps that
request tools. Discarded or cancelled generations contribute any reported counts
without contributing continuation. Final/resumed snapshots are authoritative.
Model metadata includes nullable `contextWindow`, sourced from the catalog's
optional `context_window`. No account quota or inferred token count is exposed.

The ACP projection also emits standard `usage_update` with `used` and `size` when
the latest reported context and matching model capacity are known. Its
`_meta["harness/usage"]` retains cumulative/partial details. Unknown counts or
capacity suppress that standard update rather than fabricating required values.
A model switch can defer standard usage until the final snapshot supplies its
capacity. Live updates and replay share the same projection implementation; the
custom notification remains available to existing clients.

## Tools and steering

Standard `tool_call` and `tool_call_update` updates include
`_meta["harness/tool"]` with the stable execution/message IDs, original argument
JSON, exact Harness status, preview, artifact path, truncation flag, elapsed
milliseconds and UTF-8 text offset. Terminal conversation snapshots contain the
same `tools` summaries and accepted `steering`; provider continuation is private.

`_harness/conversation/steer` accepts `{sessionId, id, text}`, where `id` is a
canonical UUID. Its `{accepted:boolean}` response is acknowledged only after
persistence, and accepted IDs are deduplicated conversation-wide. An identical
UUID/text retry is acknowledged without mutation or generation restart, including
after its original turn; a UUID bound to different text is rejected. Idle retries
check persisted history using the same identity rule. The same turn continues; clients
must not implement steering by cancelling and resending the original request.
Budget exhaustion finishes with a tool-free summary, message status `limited`,
and standard stop reason `max_turn_requests`. Standard `session/cancel` cancels
without a follow-up. All enabled tools run
automatically after any required MCP connection trust: there are no per-call
permission requests. See [tool calls](tool-calls.md) and
[MCP connections](mcp-connections.md).
