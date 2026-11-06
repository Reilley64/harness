# Research: building the Harness TUI

Researched: 2026-11-06. Scope: this Rust application, not the Pi agent harness.

Implementation decision: the project subsequently chose **Ink over ACP**, not
Ratatui. See [the current ACP presentation](../acp-presentation.md). The Ratatui
recommendation below is retained as historical research, not the active design.

## Recommendation

Build a **Ratatui + Crossterm + Tokio** presentation adapter with a functional
model/update/effects/view design. Keep the existing application ports, use cases,
DTO conversions, cache wrappers, and `fundle` composition. Start with a working
provider/model picker, then add chat when a streaming application port exists.

Ratatui supports function-based state/rendering patterns and documents the Elm
architecture. Its async example integrates Crossterm `EventStream` with
`tokio::select!`. This fits the repo's preference for functions and closure-backed
ports; a component trait hierarchy or application service object is unnecessary.
[S1][S2][S3]

This document is a proposal, not an implementation. No TUI dependencies were
added to this workspace.

## What exists today

Repository observations:

- [`presentation/cli`](../../presentation/cli/) exists but its library and dependency
  table are empty.
- [`src/main.rs`](../../src/main.rs) builds infrastructure dependencies and prints
  the model catalog. It is not an interactive application yet.
- [`application/src/use_cases.rs`](../../application/src/use_cases.rs) exposes
  `list_models` and `list_providers`, each with `#[fundle::deps]` dependencies.
- [`infrastructure/src/dependencies.rs`](../../infrastructure/src/dependencies.rs)
  composes HTTP readers, file fallbacks, memory caches, DTO conversion, and
  application-port error translation.
- [`infrastructure/src/cache.rs`](../../infrastructure/src/cache.rs) caches successful
  values in memory and falls back to persisted JSON when the source fails.
- Catalogs expose provider/model identities, not completion generation, chat
  history, credentials, tool execution, or model capabilities.

Therefore a catalog picker is implementable against existing use cases. A real
chat/agent harness requires additional application behavior; displaying an editor
and transcript alone does not supply it.

## Library selection and compatibility

Versions below were checked through published crate metadata/source, not inferred
from an older tutorial.

| Library | Investigated version | Role | Recommendation |
| --- | --- | --- | --- |
| `ratatui` | 0.30.2 | Layout, widgets, terminal drawing, headless backend | Use, with the Crossterm backend |
| `crossterm` | 0.29.0 | Input events, terminal modes, paste, resize | Use with `event-stream` |
| `tokio` | Existing workspace dependency | Async runtime, channels, task management | Reuse |
| `futures` | 0.3 family | `StreamExt::next` for terminal events | Reuse the family already used by application |
| `ratatui-textarea` | 0.9.4 | Multiline editing, cursor, selection, undo, wrapping | Add when implementing the composer |
| `tokio-util` | 0.7.20 | `CancellationToken`, optional task tracking | Add for coordinated worker cancellation, with `rt` |
| `tui-markdown` | 0.3.13 | Markdown-to-Ratatui text conversion | Optional; postpone until transcript requirements are clear |

Sources: published manifests and upstream documentation. Ratatui 0.30 splits
implementation across core/backend/widget crates; applications should still use
the main `ratatui` crate. The investigated Ratatui and its Crossterm backend require
Rust 1.88 or newer. [S4][S5][S6][S7][S8]

**Do not accidentally choose `tui-textarea` 0.7.0 for this stack.** Its published
manifest depends on Ratatui 0.29 and Crossterm 0.28. The independently maintained
`ratatui-textarea` fork uses the split Ratatui crates and matches the investigated
0.30 stack. Mixing widget generations can produce incompatible widget/event
types and duplicate dependency graphs. [S6][S9]

### Validation performed

A throwaway project outside this repository used exact versions Ratatui 0.30.2,
Crossterm 0.29.0, and ratatui-textarea 0.9.4. Running it:

1. Inserted a Crossterm key event into a textarea.
2. Rendered that textarea through `Terminal<TestBackend>`.
3. Asserted that the rendered cell contained the inserted character.
4. Type-checked an `EventStream::next()` future without polling real terminal input.
5. Inspected `cargo tree -d`; no duplicate Ratatui core or Crossterm versions appeared.

Result: passed. This verifies dependency and widget compatibility, **not** live
terminal cleanup, keyboard behavior, Windows behavior, or async application wiring.
The temporary probe is not a committed test suite.

### Alternatives

- **Cursive:** its first-party introduction uses views/dialogs and its own event
  loop. It is viable for conventional form/dialog applications, but Ratatui is the
  closer fit for our explicit state ownership, functional update loop, and custom
  streaming transcript. This preference is architectural judgment, not a claim
  that Cursive cannot handle asynchronous work. [S10]
- **Crossterm alone:** supplies terminal operations and events, but we would have
  to implement our own layout, widgets, and screen-buffer rendering. Prefer using
  it underneath Ratatui rather than rebuilding that functionality. [S4][S5]

## Crate boundaries and dependency construction

Recommended new adapter:

```text
presentation/tui/
  Cargo.toml
  src/
    lib.rs
    state.rs
    update.rs
    view.rs
    events.rs
    effects.rs
    terminal.rs
```

These are responsibilities, not a requirement to create seven files immediately.
Keep the first implementation small and split it when there is real behavior to
separate.

Dependency direction:

```text
root executable
  -> infrastructure::Dependencies::build(Settings::load()?)
  -> presentation_tui::run(application dependencies)
       -> application use cases
            -> application ports
                 -> composed infrastructure readers
```

The TUI should import `application` and `domain`, not `infrastructure`, `reqwest`,
provider DTOs, or cache implementation details. Terminal-specific state belongs
in presentation. JSON/HTTP handling and model conversion stay where they are.

A simple entry-point interface can accept the existing use-case dependency values:

```rust
pub async fn run(
    models: application::use_cases::ListModelsDeps,
    providers: application::use_cases::ListProvidersDeps,
) -> Result<(), TuiError>
```

The root can convert `&Dependencies` into each argument using the existing
`#[fundle::deps]` conversions. Add `#[derive(Clone)]` to those dependency structs
if background/retry jobs need independent owned copies; the underlying ports are
already cheap to clone. Do not borrow the root bundle into a `'static` spawned task.

Important distinction from the macro implementation: `#[fundle::deps]` generates
conversion **from an `AsRef`-capable value**; it does not automatically make its
resulting dependency struct implement `AsRef` for its fields. Do not assume one
dependency struct can be passed to another use case without the required
conversion/support. [S11]

Keep `fundle` construction in infrastructure. The root chooses which presentation
adapter to launch. Leave `presentation/cli` available for future noninteractive
commands; do not make a TUI dependency necessary for printing a catalog.

## Functional state and effects

Use data enums/structs plus free functions, not trait-based screen/controller
objects. An appropriate initial interface is:

```rust
fn update(state: &mut UiState, event: UiEvent) -> Vec<Effect>
fn render(frame: &mut ratatui::Frame, state: &UiState)
```

`update` is deterministic and performs no I/O; local mutation is confined to the
state it receives. If stricter value semantics are desired, use
`update(state, event) -> (state, effects)` without changing the surrounding design.
Rendering may use short-lived mutable widget state where Ratatui requires it,
but it must not initiate network calls or change business decisions. [S1][S3]

Suggested data:

- `UiState`: providers/models loading state, selected provider/model identities,
  current focus, active overlay, catalog errors, viewport dimensions.
- `LoadState<T>`: `Loading`, `Ready(T)`, `Failed(...)`; distinguish an empty catalog
  from a failed one.
- `UiEvent`: navigation, selection, resize, catalog loaded/failed, retry, quit.
- `Effect`: load providers, load models, cancel a job, quit.

Later add transcript, draft/editor state, request state, tool-approval state, and
streaming events. A library textarea is presentation state; it does not belong in
the domain.

Select models by **provider identity plus model identity**, not vector position or
display name. Filter using `model.provider_id`. If catalogs arrive separately,
retain models while providers are loading rather than treating them as invalid;
after both resolve, show unresolved relationships as a catalog problem.

## Async event loop

Use one task as the sole owner of UI state and terminal drawing. Other tasks
return messages; they never mutate the UI state or draw to the terminal. The
upstream async example establishes the `EventStream` + Tokio approach. [S2]

Recommended loop:

1. Initialize terminal ownership and draw a loading frame immediately.
2. Dispatch provider and model loads concurrently using the existing use cases.
   Preserve each result independently rather than failing the whole screen when
   one catalog fails.
3. Select over terminal input, worker-result messages, an optional redraw timer,
   and shutdown.
4. Translate input into `UiEvent`, update state, dispatch resulting effects.
5. Redraw only when dirty, with a capped rate while animation/streaming is active.

A starting redraw cap of 30 Hz is a project recommendation, not a Ratatui
requirement or benchmark result. An idle catalog screen needs no continuous
60-FPS rendering.

Use bounded Tokio channels for worker messages. For streaming, coalesce text
updates and drain only a bounded batch per loop turn so input and resize remain
responsive. Preserve complete content; coalescing redraws must not drop deltas.

Two correctness traps:

- **Do not recreate a use-case/network future inside every `select!` iteration.**
  A losing branch is cancelled when the other branch completes. Spawn and track a
  job, or pin a persistent future and poll it until completion. Tokio explicitly
  documents cancellation behavior and cancellation-safe operations. [S12]
- **Use one terminal input mechanism.** Crossterm says not to combine
  `EventStream` with `read`/`poll`, or read events from multiple threads. [S5]

For repeated loads, attach generation IDs to results so a late old request cannot
replace newer state. For eventual chat, attach request IDs and snapshot the model
selection when submitting; changing the picker must not relabel an active request.

## Terminal lifecycle and input

Use full-screen alternate-buffer mode first. Ratatui's default initialization
supports this. Inline viewport mode is also supported, but it should be a separate
product decision if preserving shell scrollback is important. [S13]

**Async cleanup matters:** in Ratatui 0.30.2, `ratatui::run` calls a synchronous
closure and restores the terminal immediately after that closure returns. Do not
return an async block from that closure and await it afterward: cleanup would
happen before the async application executes. [S13]

Instead initialize the terminal, await the event loop while the terminal is owned,
and restore it afterward. Use a small RAII cleanup guard so dropped futures and
early returns also restore the terminal. Ratatui initialization installs a panic
hook; custom paste/mouse modes must also be paired with cleanup. This private
resource guard does not require an object-oriented application architecture.
Abrupt termination such as SIGKILL cannot be guaranteed to run cleanup. [S13]

Additional decisions:

- Route `tracing` output to a file or a dedicated UI log buffer, not stdout/stderr
  while the TUI owns the screen.
- Enable bracketed paste for the composer and handle `Event::Paste` as text, not
  as a sequence of submission/navigation commands. The input feature and explicit
  terminal-mode commands are separate concerns. [S5]
- Route keys by focus; `q` must type a character while editing, not always quit.
- Handle key press/repeat/release deliberately; avoid duplicate submissions.
- For the composer, start with Enter for newline and an explicit send shortcut
  such as Ctrl+S, then confirm the preferred UX. Do not assume all terminals
  distinguish Shift+Enter or Ctrl+Enter without enhanced keyboard support.
- On resize, recompute layout and redraw. Provide a small-terminal fallback.
- Use textarea editing rather than writing byte-index cursor logic. Its upstream
  feature set includes multiline editing, undo/redo, wrapping, and selection. [S6]

For shutdown, request cancellation, stop new jobs, and wait for tracked jobs with
an appropriate bound. Tokio's shutdown guidance separates detecting shutdown,
notifying tasks, and waiting for completion. Escape/cancel is a presentation
choice; a `CancellationToken` coordinates the actual worker shutdown. [S14]

## Initial screen and implementation stages

### Stage 1: catalog browser

Implement only behavior supported by today's application:

```text
+ Harness --------------------------------------------+
| Providers          | Models                         |
| > ChatGPT          | > GPT-6 Luna                    |
|                    |   GPT-6.1 Terra                 |
|                    |   GPT-6.1 Sol                   |
|                    |   GPT-6 Astra                   |
+-----------------------------------------------------+
| Loading / error details / selected provider + model |
| Tab: focus   arrows: select   r: retry   Ctrl+Q: quit |
+-----------------------------------------------------+
```

The names are illustrative renderings of the current catalog, not a new model
registry. Handle empty catalogs, one-catalog failure, unknown provider references,
and selection changes when catalogs reload.

**Retry is not force refresh today.** The existing memory wrapper may serve a
cached success until TTL expiry. If the product needs “fetch fresh now,” design
an explicit invalidation/bypass mechanism; do not label a normal cached read as
force refresh. Likewise the ports return values without cache provenance, so the
UI cannot honestly show “offline/stale cache” without additional metadata.

### Stage 2: composer and transcript shell

Add ratatui-textarea, focus handling, scrolling, and a transcript view. Either
label this as a nonfunctional preview or wait for a real conversation port before
enabling send. Consider `tui-markdown` only after plain text behavior works. [S8]

### Stage 3: streaming conversation

Add a provider-neutral application port yielding structured events, e.g.
`Started`, `TextDelta`, `Completed`, `Failed`, with a clear cancellation policy.
Keep OpenAI request/response DTOs and SSE parsing in infrastructure. OpenAI's
streaming guide documents SSE and events including `response.output_text.delta`
and `response.completed`; do not leak those transport payloads into widgets. [S15]

A response stream is distinct from an agent loop. Tool calls, filesystem/shell
execution, approvals, limits, credential handling, conversation persistence, and
retries need separate domain/application decisions before this becomes a coding
agent harness. Do not automatically retry an interrupted generation without a
policy for duplicate work, cost, and partial output.

## Validation strategy for a later implementation

No tests were added by this research. If requested during implementation:

- Exercise state transitions with scripted events, without a terminal/network.
- Use Ratatui `TestBackend` for deterministic frame assertions at normal and small
  sizes; first-party docs expose direct buffer and line assertions. [S16]
- Inject closure-backed application ports to simulate delayed catalogs, failures,
  and eventual streams; no real provider credentials needed.
- Manually smoke-test real terminal restoration on normal exit, failure, panic,
  resize, paste, and cancellation. Headless rendering alone does not verify these.

## Decisions to confirm before coding

1. Start with the catalog picker, or design the chat port before presenting a UI?
2. Full-screen alternate buffer or an inline, scrollback-preserving experience?
3. Separate `presentation/tui` adapter (recommended) or evolve the empty CLI crate?
4. Desired composer send/newline bindings and cancellation/quit behavior?
5. Should provider/model selection persist between runs, and where?

## Primary sources

- **S1:** [Ratatui: The Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/).
- **S2:** [Ratatui official async example](https://github.com/ratatui/ratatui/blob/main/examples/apps/async-github/src/main.rs).
- **S3:** [Ratatui state-pattern examples](https://github.com/ratatui/ratatui/tree/main/examples/concepts/state).
- **S4:** [Ratatui 0.30.2](https://docs.rs/crate/ratatui/0.30.2), [architecture](https://github.com/ratatui/ratatui/blob/main/ARCHITECTURE.md), [Crossterm backend 0.1.2](https://docs.rs/crate/ratatui-crossterm/0.1.2).
- **S5:** [Crossterm 0.29 event module](https://docs.rs/crossterm/0.29.0/crossterm/event/index.html), [features](https://docs.rs/crate/crossterm/0.29.0/features).
- **S6:** [ratatui-textarea upstream README](https://github.com/ratatui/ratatui-textarea), [0.9.4 manifest](https://docs.rs/crate/ratatui-textarea/0.9.4/source/Cargo.toml).
- **S7:** [tokio-util 0.7.20 features](https://docs.rs/crate/tokio-util/0.7.20/features), [CancellationToken](https://docs.rs/tokio-util/0.7.20/tokio_util/sync/struct.CancellationToken.html).
- **S8:** [tui-markdown 0.3.13](https://docs.rs/tui-markdown/0.3.13/tui_markdown/), [manifest](https://docs.rs/crate/tui-markdown/0.3.13/source/Cargo.toml).
- **S9:** [tui-textarea 0.7.0 manifest](https://docs.rs/crate/tui-textarea/0.7.0/source/Cargo.toml).
- **S10:** [Cursive first-party README](https://github.com/gyscos/cursive).
- **S11:** [Fundle dependency-macro implementation](https://docs.rs/crate/fundle_macros_impl/0.4.1/source/src/deps.rs).
- **S12:** [Tokio `select!`: concurrency, fairness, cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html).
- **S13:** [Ratatui 0.30.2 initialization source](https://docs.rs/crate/ratatui/0.30.2/source/src/init.rs).
- **S14:** [Tokio: graceful shutdown](https://tokio.rs/tokio/topics/shutdown).
- **S15:** [OpenAI: streaming API responses](https://developers.openai.com/api/docs/guides/streaming-responses).
- **S16:** [Ratatui TestBackend](https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.TestBackend.html).

Library/API facts above are sourced; proposed module placement, keybindings,
redraw rate, state ownership, and implementation stages are design recommendations.
