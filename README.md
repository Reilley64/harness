# Harness

A local Bun/React/Ink coding assistant with a Rust-owned model/tool loop and
durable conversations. Factory capabilities are delivered incrementally;
[approved work](docs/approved-work.md), [native selection](docs/native-decider.md)
and [managed Research](docs/managed-dispatch.md) document their actual boundaries.
No runtime sandbox, live-provider qualification or whole-MVP completion is implied.

## Terminal interface

With [Mise](https://mise.jdx.dev/), install the pinned tools and launch:

```sh
mise trust
mise install
mise run
```

Invoke `/path/to/harness/bin/harness` from another repository to preserve that
workspace. Mise owns Rust build/start; Bun runs TSX directly. The launcher owns
its ACP child; do not start a second backend separately. `cargo run` starts only
the stdio ACP backend and reserves stdout for JSON-RPC.

For the included development catalogs, use a separate terminal:

```sh
python3 -m http.server 3000 --bind 127.0.0.1 --directory server
```

Use `/auth` and `/model` for explicit connection/model selection. Connecting does
not make a paid inference probe. Startup selects repository Setup or Griller
without inference; sending begins real work. [Frontend guidance](docs/frontend.md)
owns keys, paste, navigation, settings and inspection. Escape navigates; explicit
Stop is separate. Tools are [OS-user trusted](docs/security.md), not sandboxed.

## Conversations and app data

```sh
./bin/harness --resume <conversation-id>
```

The ID is a canonical lowercase UUID. Loading history is not execution Resume.
Read [persistence and recovery](docs/conversations/recovery.md) before continuing
interrupted work; uncertain effects are not automatically replayed.
`APP_DATA_PATH` overrides the shared `~/.harness` storage root. Local files contain
sensitive plaintext; [security](docs/security.md) owns protection and limitations.

## ChatGPT sign-in

See [provider connections](docs/provider-connections.md#chatgpt-flow).

## OpenRouter workers

See [OpenRouter connection and origin binding](docs/provider-connections.md#openrouter-flow),
[model permission settings](docs/model-allowlist.md), and
[worker continuation](docs/conversations/providers.md).

## Provider configuration

See [catalog endpoints and environment overrides](docs/catalog-configuration.md#provider-configuration).

## Model configuration

See [model schema and static catalog provenance](docs/catalog-configuration.md#model-configuration).

## Reading and caching

See [bounded catalog caches and fallback](docs/catalog-configuration.md#catalog-caches).

## Infrastructure layout

Read [coding standards](CODING_STANDARDS.md) and the applicable scoped instructions
before editing. Dependencies point inward; port-owned adapters and composition
are described there, rather than maintained as a second source inventory here.
[ACP](docs/acp-presentation.md) owns wire contracts; [frontend development](docs/frontend/implementation.md)
owns presentation architecture.

## Continuous integration

Use focused local checks for changed work; CI owns whole-project build/test/check
suites. [CI notes](docs/ci.md) list exact tasks, pins, coverage and PR policy.
No permanent tests or test-only dependencies are added.
