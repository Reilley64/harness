# Infrastructure

Follow [the repository coding standards](../CODING_STANDARDS.md), including
cancellation-token-last signatures, single-item imports, and workspace Clippy settings.

## Port adapters

- Every application port implemented in this crate has exactly one matching
  module: `application::ports::ReadFile` is implemented in
  `src/read_file/mod.rs`, with a `read_file` constructor.
- Each adapter module implements one port. Keep its DTOs, settings, and
  provider-specific helpers inside that module when they belong only to it.
- Share technical helpers privately when multiple adapters need them; shared
  helpers are not alternative places to implement ports. Preserve the matching
  module and constructor for every adapter.
- Factory ports follow the same rule. A factory synchronously selects an owned,
  provider-bound adapter; the selected adapter does not switch on provider again.
- `dependencies.rs` composes adapters with `fundle` and converts their results to
  application port errors. Put execution, parsing, persistence, and backend
  selection in their owning modules, not in composition closures.

## Port-owned representations

- Each port owns its wire/storage representation inside its matching adapter
  module. Convert through domain types between ports; do not import another
  port's representation or introduce a shared DTO module.
- Use role-specific names instead of the `Dto` postfix: `Arguments` for tool
  inputs, `Response` for remote payloads, and `Record` for persisted data.
- Keep nested representation details private to the same owning port. Share
  technical functions and domain values, not serialized representation types.
- Keep Serde derives and external field naming on these port-owned types;
  conversion to domain values happens inside the owning adapter module.
- Keep only the serialization/conversion directions the adapter uses. Delete
  unused directions rather than merging independently owned records.

## Interfaces and effects

- Import crate-local modules/types with `use`; use those imported names in bodies,
  type signatures, and fields. Keep `crate::` paths in import declarations only.

- Domain tool identities and invocations are enums. Decode external names and
  argument JSON into typed domain values before execution; keep domain types
  Serde-free. Wire DTOs and serialization belong in infrastructure.
- Constructors borrow paths/settings where practical and return owned closures;
  use Rust 2024 `+ use<>` when needed to make capture behavior explicit.
- Preserve original errors through `caused_by` and public boxed conversions.
  Sanitize diagnostics at presentation seams instead of discarding causes.
- Use Tokio filesystem/network/process I/O and
  `tokio_util::sync::CancellationToken` directly. Use `spawn_blocking` for blocking
  locks, temporary-file commits, directory sync, and bounded CPU-heavy parsing.
- Derive all Harness-owned storage paths from `app_data_path`. Load embedded
  settings with environment overrides; do not introduce runtime TOML files.
- Tool adapters execute operations and return typed outcomes. The application
  coordinator owns conversation checkpoints, ordering, budgets, and continuation.
  Keep OAuth credentials out of proactive tool environment/prompt injection.

## Completion check

For every port added or changed, verify its matching adapter module, constructor,
and composition entry. Run workspace build, strict Clippy, formatting, and the
relevant temporary external probes. Preserve unrelated working-tree changes and
add no permanent tests or test-only dependencies.
