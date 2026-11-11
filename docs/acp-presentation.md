# ACP presentation

The terminal frontend is implemented in `presentation/tui` with React and Ink,
using Bun as its package manager and runtime. It follows
[atomic design](frontend.md), with rendering separated from hooks and ACP transport.
It spawns the Rust `harness` executable, whose `presentation/acp` adapter uses
the official Rust ACP SDK. The frontend uses the official TypeScript SDK.
Both select stable ACP v1; no unstable provider/auth extensions are enabled.

Model catalog and historical model summaries expose a typed `capabilities` array
with `chat` and `decisions` values. Missing historical metadata remains an empty
array, not implied chat support. `blocked` reflects current backend admission;
historical summaries never authorize inference.

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
`openai` (Continue with ChatGPT) and `openrouter` (Continue with OpenRouter)
authentication methods, and includes:

```json
{
  "_meta": {
    "harness/providerCatalog": true,
    "harness/modelCatalog": true,
    "harness/modelSelection": true,
    "harness/tools": true,
    "harness/steering": true,
    "harness/agents": true,
    "harness/creationMap": true,
    "harness/guidance": true,
    "harness/approvedWork": true,
    "harness/setup": true,
    "harness/mcp": true,
    "harness/nativeDecider": true,
    "harness/modelAllowlist": true,
    "harness/usage": true,
    "harness/children": true
  }
}
```

The frontend checks both the negotiated protocol version and the extension marker
before requesting providers, and checks the model marker before requesting models.

## Repository-agent extension

See [Repository-agent extension](acp/conversations.md#repository-agent-extension).

## Creation-map extension

See [the bounded, read-only durable-agent inventory](acp/creation-map.md).
Creation arrows describe recorded creators, never execution ownership or Stop scope.

## Approved-work extension

See [Approved-work extension](acp/conversations.md#approved-work-extension).

## Native Decider extension

See [Native Decider extension](acp/conversations.md#native-decider-extension).

## Guidance extension

See [Guidance extension](acp/conversations.md#guidance-extension).

## MCP configuration extension

See [MCP configuration extension](acp/mcp.md#mcp-configuration-extension).

### MCP OAuth challenge correlation

See the [exact challenge/correlation contract](acp/mcp.md#mcp-oauth-challenge-correlation).

### MCP cancellation and supported transports

See [cancellation, owned commits and shutdown](acp/mcp.md#mcp-cancellation-and-supported-transports).

## Provider catalog extension

See [Provider catalog extension](acp/catalogs-and-connections.md#provider-catalog-extension).

## Model catalog extension

See [Model catalog extension](acp/catalogs-and-connections.md#model-catalog-extension).

## Model permission settings

See [Model permission settings](acp/catalogs-and-connections.md#model-permission-settings).

## Saved model selection

See [Saved model selection](acp/catalogs-and-connections.md#saved-model-selection).

## Provider connection creation

See [Provider connection creation](acp/catalogs-and-connections.md#provider-connection-creation).

## Conversations

See [Conversations](acp/conversations.md#conversations).

## Provider waits and accounting

See [provider waits](provider-waits.md) for `_harness/usage/read`, trusted
`_harness/provider-retry/update`, live wait metadata, identities, currencies,
unknown values, limits and the separate native no-retry policy.

## Token usage and workspace

See [Token usage and workspace](acp/conversations.md#token-usage-and-workspace).

## Tools and steering

See [Tools and steering](acp/conversations.md#tools-and-steering).

## Child coordination extension

See [dispatch, captured joins, status and source-linked results](acp/children.md).

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
