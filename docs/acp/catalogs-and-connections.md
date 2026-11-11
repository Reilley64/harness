# ACP catalogs and provider connections

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
    "providers": [{ "id": "openai", "name": "ChatGPT" }, { "id": "openrouter", "name": "OpenRouter" }]
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

## Model permission settings

The `harness/modelAllowlist` extension exposes ordinary application policy reads
and replacements. The catalog adds nullable `blocked` local-admission diagnostics;
catalog presence alone is not eligibility or account entitlement. See
[model allowlist](../model-allowlist.md) for request shapes, errors and cancellation.
Saved model snapshots do not establish current permission.

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
ChatGPT calls standard ACP `authenticate` with `methodId: "openai"`; OpenRouter
uses `methodId: "openrouter"` and the same safe challenge/cancellation envelope.
The adapter
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
[provider connections](../provider-connections.md) for OAuth and credential storage.
