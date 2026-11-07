# Harness

See [CODING_STANDARDS.md](CODING_STANDARDS.md) for repository conventions and
formatting/lint configuration.

## Continuous integration

GitHub workflows run Rust tests, doctests, formatting, Clippy, TUI checks and
release builds on pushes and pull requests. All jobs currently run on Linux.
See [CI notes](docs/ci.md) for coverage, toolchain pins and pull request policy.
See [security and privacy](docs/security.md) for data flows, secret scanning and
the limits of unsandboxed tool access.

## Terminal interface

With [mise](https://mise.jdx.dev/), install the pinned tools and run:

```sh
mise trust
mise install
mise run
```

Mise pins Rust and its components, Bun, and the lint tools in `mise.toml`,
installs frontend dependencies, builds Rust, and starts
the TUI with direct terminal access. `mise run tui` and `mise run dev` are aliases.
The TUI starts ACP as a background child connected over stdio and shuts it down
on exit; do not launch a second ACP process separately.

Invoke `/path/to/harness/bin/harness` from another project to delegate to the same
Mise launch task while preserving that project's directory as the workspace.
`mise run` also preserves its invocation directory. Explicit resume retains the
conversation's saved workspace. Rust build/start scripts live in Mise, not the
TUI's `package.json`. Ink
spawns the executable as a separate ACP agent and communicates over stdio. Bun
runs the TypeScript/TSX frontend directly; no Node.js or `tsx` launcher is needed.

The frontend follows atomic design: atoms, molecules, organisms, a layout template,
and a screen, with asynchronous behavior in hooks and ACP transport isolated from
rendering. See [the frontend notes](docs/frontend.md) for the structure and the
planned TanStack AI integration.

To serve the included catalogs locally, run this in a separate terminal:

```sh
python3 -m http.server 3000 --bind 127.0.0.1 --directory server
```

Type `/auth` and press Enter to open the provider picker. Use the arrow keys to
select, Enter to connect, Escape to close, and `r` to retry a failed catalog
load. Selecting ChatGPT opens browser sign-in for an eligible ChatGPT plan.
OpenRouter uses protected browser PKCE and a separately billed API key; connecting
performs no paid inference. Compatible OpenRouter workers come from its public
catalog, not a promise of account entitlement.
While waiting, Escape cancels and `o` reopens the sign-in page. Ctrl+Q or Ctrl+C
exits. After signing in and choosing a model, enter a message to start a streamed,
automatically saved conversation. While it works, keep typing and press Enter to
steer: Rust durably accepts your instruction and continues the same turn. In-flight
inference restarts; a running tool finishes before the remaining batch is skipped.
Escape closes or navigates without stopping work and preserves drafts. Use
`/stop`, or Ctrl+X from the agent map, then confirm to stop the active leaf run.
The nine automatic tools and their limits are described in [tool calls](docs/tool-calls.md).
Whole tool panels turn green on success or red on failure; Ctrl+O expands output.

Type `/model` to open the searchable model picker. Type to filter by model name,
ID, or provider; use ↑/↓ to move, ←/→ to change the highlighted model's supported
effort, Enter to confirm, and Escape to cancel. Ctrl+U clears search and Ctrl+R
reloads the catalog. The selected provider/model ID and raw effort ID appear at
the footer's bottom-right (for example, `openai/gpt-6.1-sol · extra_high`) and
apply to the next message. The last selection is saved for fresh sessions. Choosing a model alone does not start inference or
create a conversation. The model/effort used for each reply is saved with it.

The pricing section shows catalog USD rates per million input, cached-input,
and output tokens. Missing prices display `—`, not zero. These are catalog
rates, not ChatGPT subscription billing or an estimate of the current request.

The prompt preserves multiline pasted text without submitting. Alt+Enter or
Ctrl+J inserts a newline when the terminal cannot report Shift+Enter.
Command prefixes show dim inline suggestions for `/auth`, `/model`, `/quit`, and `/exit`.
Press Tab to complete a suggestion, then Enter to run it. `/quit` and `/exit`
close the TUI and its ACP child process.

The Rust `presentation/acp` adapter uses stable ACP v1. The custom
`_harness/providers` and `_harness/models` requests call their respective read
use cases. Initialization advertises them through `_meta["harness/providerCatalog"]`
and `_meta["harness/modelCatalog"]`, along with separate `openai` and `openrouter`
authentication methods. Standard ACP `authenticate` calls the
`create_provider_connection` use case. Tokens remain in Rust; only browser
instructions and safe connection information reach Ink.

`cargo run` now starts the ACP stdio agent directly; it does not render the UI.
Keep stdout reserved for ACP JSON-RPC messages. See
[the ACP presentation notes](docs/acp-presentation.md) for the wire interface.

## Conversations and app data

Startup is fresh by default, with the last selected model/effort but no old
messages. The first message creates a conversation. PgUp/PgDn browse history;
Escape returns to the latest text. Resume a saved conversation explicitly without
regenerating anything (its ID is the filename under `conversations/`):

```sh
./bin/harness --resume <conversation-id>
```

The ID must be a canonical lowercase UUID, not a filename or path. Missing,
invalid, corrupt, or nonexistent conversations fail clearly rather than falling
back to a fresh session. History loads without credentials; sending still
requires the original provider connection and an available model. There are no
conversation commands or conversation picker.

All local storage derives from one setting, `app_data_path`, defaulting to
`~/.harness`. Override it with `APP_DATA_PATH`; a leading `~` expands using
`HOME`. No runtime TOML file is loaded.

```text
~/.harness/
├── auth/
│   ├── openai.json
│   ├── openai.lock
│   ├── openrouter.json
│   └── openrouter.lock
├── conversations/
│   ├── <conversation-id>.json
│   └── <conversation-id>.lock
├── preferences/
│   └── model-selection.json
├── models.json
└── providers.json
```

Conversations contain plaintext prompts/replies in owner-only files, not tokens.
The user message is saved before inference. Completion, cancellation, and failure
save the final/partial assistant text; a crashed active reply resumes as
interrupted. Only completed turn pairs enter future model context. See
[conversations](docs/conversations.md) for the workflow and durability boundaries.

## ChatGPT sign-in

Harness uses OpenAI's documented Sign in with ChatGPT flow for open-source/local
applications, including dynamic client registration. No API key, client secret,
Codex installation, or copied Codex client ID is needed. Account eligibility and
preview restrictions still apply.

Credentials are stored under `~/.harness/auth/`, using a private directory and
owner-only files. Existing credentials are replaced only after successful
validation and persistence. File-backed credential protection currently requires
Unix; Windows sign-in fails explicitly until secure storage is implemented there.

Defaults are embedded in `application.toml`. Environment overrides include:

- `APP_DATA_PATH`: shared app data directory, default `~/.harness`.
- `AUTHENTICATION__OPENAI__OPEN_BROWSER`: whether Rust opens the browser automatically.
- `AUTHENTICATION__OPENAI__TIMEOUT_SECONDS`: login timeout, default 600 seconds.
- `AUTHENTICATION__OPENAI__DYNAMIC_CLIENT_ID`: initial-registration client ID,
  default `dynamic_agent_client`.
- `AUTHENTICATION__OPENAI__RESOURCE`: OAuth resource, default
  `https://api.openai.com/v1`.
- `AUTHENTICATION__OPENAI__SCOPES`: space-separated requested OAuth scopes.
- `AUTHENTICATION__OPENAI__ISSUER`: normally `https://auth.openai.com`; override only
  for a trusted environment. HTTP is accepted only on loopback for local fixtures.

Sign-in, returning sign-in, text inference, and on-demand token refresh are
implemented. There is no background refresh loop, account switching, or logout.
See [provider connections](docs/provider-connections.md) for security details.

## OpenRouter workers

Choose OpenRouter in `/auth`, then an allowed compatible model/effort in `/model`.
The Rust provider factory uses the existing Responses stream/tool coordinator.
It requires requested parameters, disables gateway provider fallback/plugins and
rejects hosted routers and unknown protocol/tool/context capabilities. Provider
or model switches on later prompts preserve ordinary history, effects and usage,
without forwarding incompatible private reasoning or response-item/call IDs.
OpenAI and OpenRouter credential stores remain separate.

`MODELS__OPENROUTER_URL` defaults to `https://openrouter.ai/api/v1/models`.
`AUTHENTICATION__OPENROUTER__ISSUER` defaults to `https://openrouter.ai`; the
inference origin in `REPLIES__OPENROUTER__BASE_URL` must match it and the stored
issuer. Both auth and reply sections have a nonzero `TIMEOUT_SECONDS` default of
600; auth also has `OPEN_BROWSER`. Only HTTPS or explicit loopback HTTP fixtures
are supported. Public catalog/PKCE fixtures are not live account, billing or
quality proof. Native Jev decision inference is a later, separate capability.

See [provider connections](docs/provider-connections.md),
[model permission settings](docs/model-allowlist.md), and
[conversation continuation](docs/conversations.md) for the full contracts.

## Provider configuration

Defaults are embedded from `application.toml`. Runtime overrides use environment
variables only, with **no prefix** and `__` separating groups:

```sh
PROVIDERS__URL=https://your-endpoint.example/providers.json \
PROVIDERS__MEMORY_TTL_SECONDS=300 \
APP_DATA_PATH=~/.harness \
./bin/harness
```

The default URL is `http://localhost:3000/providers.json`. The endpoint should
return a JSON array:

```json
[{ "id": "example", "name": "Example Provider" }]
```

## Model configuration

Models use the same reader and cache composition, with defaults embedded from the
`[models]` section of `application.toml`. Overrides are `MODELS__URL`,
`MODELS__MEMORY_TTL_SECONDS`, and the shared `APP_DATA_PATH`. The URL defaults to
`http://localhost:3000/models.json`, TTL to 300 seconds, and cache to
`<app_data_path>/models.json`.

The endpoint returns an array of objects with `id`, `name`, and `provider_id`:

```json
[
  {
    "id": "example-model",
    "name": "Example Model",
    "provider_id": "openai",
    "efforts": ["low", "medium", "high", "extra_high"],
    "pricing": {
      "input": 1.25,
      "cached_input": 0.25,
      "output": 10
    }
  }
]
```

The `read_models` use case reads through the `ReadModels` port. JSON serialization
stays in the infrastructure DTO; the dependency builder converts DTOs into domain
models. `efforts` is optional and defaults to no effort control; the picker only
offers the listed levels, preferring medium when supported. `pricing` and its
individual fields may be omitted or null. Prices above are illustrative USD per
million tokens, not published rates for an actual model. A numeric zero means
free; missing means unavailable.

`server/models.json` is a static catalog, not a live provider inventory. Its prices
are OpenAI's published Standard, short-context USD rates per million tokens,
verified on 2026-11-06. Long-context requests, processing modes, cache writes, and
other fees may differ. See [pricing sources and limitations](docs/research/model-pricing.md).

## Infrastructure layout

Infrastructure is organized by the application port it implements:

```text
infrastructure/src/
├── create_conversation/
├── create_provider_connection/
├── create_provider_connection_factory/
├── create_reply/
├── create_reply_factory/
├── lock_conversation/
├── read_conversation/
├── read_models/
├── read_provider_connection/
├── read_providers/
├── update_conversation/
├── conversations/
├── storage.rs
├── cache.rs
├── dependencies.rs
├── lib.rs
└── settings.rs
```

Each adapter owns its DTOs, settings, and implementation details. The factory
selects the provider-specific connection function. Cache helpers are shared;
`dependencies.rs` composes adapters into application ports and converts DTOs into
domain values. `PresentAuthorization` is implemented by the ACP presentation,
not infrastructure.

The `authentication`, `models`, `providers`, and `replies` sections configure adapter
behavior. Storage locations are always derived from `app_data_path`; there are
no separate credential or catalog-cache path settings.

## Reading and caching

- The application exposes a closure-backed `ReadProviders` port. Infrastructure
  constructs the HTTP reader with a function. The `fundle` builder composes generic
  `with_memory_cache` and `with_file_fallback` wrappers around it before passing the
  port to the `read_providers` use case.
- Port clones share a memory cache, valid for five minutes by default.
- On a miss or expiry, fetch the URL (10-second timeout). Concurrent reads share
  one refresh.
- Successful responses update memory and atomically replace the disk cache.
  Persistence failures emit a tracing warning but do not discard fetched data.
- Connection, HTTP, and JSON errors fall back to the disk file regardless of age.
  Fallback results also enter memory for the configured TTL.
- If both sources fail, return an error. An empty array is a valid result.
- The default disk path is `~/.harness/providers.json`; a leading `~` is expanded
  from `HOME`. Relative configured paths are relative to the working directory.

Use `mise run build`, `mise run test`, and `mise run check` for local validation.
`mise run lint` runs linting only; `mise run format` formats Rust and the TUI.
`mise run build:release` performs the same release build as GitHub CI.
