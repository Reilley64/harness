# Coding standards

## Architecture

- Dependencies point inward: domain owns pure rules; application orchestrates
  use cases through ports; infrastructure implements ports; presentation maps
  protocols and UI to use cases. Composition wires implementations.
- Prefer deep modules with small interfaces that hide substantial logic. Introduce
  seams for independent policy or effects, not merely to shorten a file. Use the
  deletion test when proposing abstractions: what complexity actually disappears?

## Rust structure

- Use functional functions and closure-backed application ports, composed with
  `fundle`. Keep orchestration in application and external effects in infrastructure.
- Keep one domain type per snake_case file, with its implementations alongside it.
  Re-export types from `domain/src/lib.rs`; keep domain types Serde-free.
- Use domain enums for tool identities and typed invocations.
- Give each application port its own file and each implemented infrastructure port
  a matching module and constructor. Keep composition separate from implementation.
- Each port owns its wire/storage representations; convert through domain types
  between ports rather than sharing DTOs. Use role-specific names:
  `Arguments`, `Response`, and `Record`, rather than a `Dto` postfix.
- Name public application use cases with CRUD verbs. Expose `call`,
  `Dependencies`, `Error`, and `Output`; keep use-case arguments inline instead of
  introducing an `Input` wrapper.
- Apply scoped `AGENTS.md` instructions for the layer being changed.

## Signatures and imports

- A `CancellationToken` parameter is always the last argument. Apply this to
  functions, closures, port signatures, and their call sites, including borrowed
  tokens. Use `tokio_util::sync::CancellationToken` directly, without a wrapper.
- Allow `clippy::too_many_arguments` throughout this Cargo workspace. Prefer clear
  explicit arguments over parameter objects added solely to satisfy that lint.
  Every member inherits the repository's `[workspace.lints]` configuration.
- Put exactly one imported item on each `use` declaration. Run rustfmt to split
  grouped imports using `imports_granularity = "Item"`.
- Import crate-local names with `use`; keep `crate::` paths in import declarations,
  not bodies, signatures, or fields. Application code refers to domain types as
  `domain::Type`.
- Keep dependencies alphabetized. Put standard derives before qualified external
  derives, such as `#[derive(Debug, Clone, serde::Deserialize)]`.
- Constructors borrow paths/settings where practical and return owned closures.
  Use Rust 2024 `+ use<>` when needed to express precise captures.

## Semantic spacing

Use blank lines to separate logical phases, such as validation, state preparation,
external effects and result construction. Keep closely related statements and
checks together rather than separating every statement. Separate top-level
definitions and distinct hook lifecycles. Do not insert spacing inside literals or
change evaluation order for layout. Formatters normalize syntax; they do not infer
these semantic groups.

## Errors, effects, and storage

- Use `#[ohno::error]` for application errors and boxed `From` conversions at
  adapter seams. Preserve original errors with `caused_by`; sanitize diagnostics
  at the ACP boundary rather than discarding causes.
- Use Tokio filesystem, network, and process I/O. Move blocking locks,
  temporary-file commits, directory sync, and bounded heavy parsing to
  `spawn_blocking`.
- Load embedded `application.toml` with unprefixed environment overrides using
  `__`. Keep runtime TOML files out of configuration loading.
- Derive all Harness-owned credential, cache, conversation, preference, and
  artifact paths from `app_data_path` (default `~/.harness`,
  override `APP_DATA_PATH`).
- Keep credentials private and persistence bounded and atomic. Persist tool
  execution starts before effects and results before model continuation; represent
  uncertain outcomes honestly rather than replaying them.
- Keep source free of explanatory comments; express intent through names,
  module boundaries, and documentation.

## Frontend and verification

- Use Bun, React, and Ink. Follow atomic component organization; isolate transport
  in services and asynchronous UI behavior in hooks.
- Preserve unrelated staged and unstaged work.
- Treat module interfaces as the test surface. Verify behavior with temporary
  external probes; keep implementation helpers private. Add no permanent tests
  or test-only dependencies.
- Follow the [root verification policy](AGENTS.md#verification) for local versus
  CI scope. See [docs/ci.md](docs/ci.md) for available tasks and coverage limits.

The repository pins nightly Rust in `mise.toml` because rustfmt's
`imports_granularity` setting requires nightly. The formatting and Clippy
configuration is repository-local; it does not change machine-wide settings.
