# Frontend development

## Atomic design

- **Atoms** own terminal visual primitives and invariants. `TerminalText` removes
  control characters from displayed strings; `Panel` owns shared panel styling.
- **Molecules** combine primitives into prompt editing, search, model/provider
  options, model pricing, and animated work status. The prompt molecule owns
  multiline editing, paste normalization, completion, and focus. Enter submits;
  Shift+Enter (where reported by the terminal), Alt+Enter, or Ctrl+J inserts a newline.
  Bracketed paste preserves line breaks, blank lines, tabs, and outer whitespace,
  normalizing CRLF/CR to LF. The editor grows to a bounded viewport without
  truncating the draft; arrows navigate logical and wrapped lines. Rendering and
  movement share one cached grapheme/cell geometry implementation in
  `services/prompt-layout.ts`; left/right and deletion preserve whole graphemes,
  including combining accents and joined emoji. Keyboard interaction stays in the
  molecule rather than a new asynchronous hook.
- **Organisms** assemble richer views. The provider picker renders loading,
  failure, empty, and populated states and keeps the selected option in view.
- **Templates** own layout slots and terminal-size constraints, not commands or I/O.
- **Screens** are atomic design's pages: they compose the layout, command handling,
  keyboard routing, and hooks for the current terminal workflow.
- **Hooks** own asynchronous state and lifecycle. The picker hook hides request
  cancellation, dismissal, stale-result suppression, navigation, and selection.
- **Services** own transport. `connectAgent` hides subprocess management, ACP
  negotiation, request timeouts, response validation, and shutdown.

Rendering modules receive data and callbacks. Presentation-only hooks may own
viewport measurement and terminal input; agent transport stays outside rendering. Higher visual layers compose lower ones and may use
Ink primitives directly; there is no need to wrap every `Box` or `Text`.
Shared presentation types do not depend on transport or React.

The screen receives the agent as a dependency. Only the entry point creates the
agent, starts rendering, and shuts it down. No visual module launches a process or
performs network I/O.

## Bun

Mise installs the pinned Bun, Rust and lint tools:

```sh
mise install
mise run
mise run typecheck
mise run lint
mise run format
```

The default task depends on `build:agent` and `install:tui`, then runs the TSX entry
point from the invoking workspace. `bin/harness` captures Mise's invocation
directory, explicit `INIT_CWD`, or its shell cwd and delegates to the same task,
allowing launch from another project without changing its bound workspace.
Rust build and launch commands are owned by Mise; the package retains only
frontend check/format scripts. The typecheck task uses Bun's runtime override
for the TypeScript executable.
Biome owns formatting, import organization, and linting, with recommended React,
project, and type-analysis domains. Its project/type scanning complements `tsc`;
it does not replace typechecking. Run `mise run check:tui` for read-only validation,
`mise run format` for formatting, or the package's `check:fix` for safe lint fixes. Control-character regex checks
are disabled only in terminal sanitization/input modules, whose patterns
intentionally inspect control bytes. Unrelated framework and test domains are not enabled for this Ink application.
Node-compatible APIs are used for child processes and stdio streams; no Bun-only
transport implementation is necessary.

## TanStack AI

TanStack AI can fit this architecture without replacing ACP or moving provider
access into the frontend. A temporary compatibility probe verified
`@tanstack/ai-react` 1.0.2, `@tanstack/ai` 1.2.0, Ink 8, React 19.3.1, and
Bun 1.4.3 together: `useChat` consumed a custom streaming adapter and produced
an assistant message without React DOM. This was a synthetic stream, not an
end-to-end ACP chat test.

TanStack AI remains **deferred and is not installed**. The real ACP chat path now
uses a small conversation hook, with Rust authoritative for durable history.
Introducing another client-state layer is not necessary for this implementation.

If introduced later:

1. Add a custom TanStack AI `ConnectionAdapter` under `services/`. Its
   subscribe/send form fits ACP's request/notification interface; its connect
   form can instead expose a per-turn async iterable. Neither needs an HTTP
   endpoint or a second agent connection.
2. Translate ACP message/tool updates into TanStack AI `StreamChunk` events and
   map completion, errors, and cancellation explicitly. Send only the new turn
   to the existing ACP session rather than replaying the client's whole history.
3. Use `useChat` in a hook, with a stable `threadId` mapped to the ACP session.
   Use `persistence: false` initially; do not enable browser storage in a terminal
   runtime. Bridge stop/cancellation to ACP session cancellation.
4. Render the hook's messages through chat molecules/organisms. Keep provider
   SDKs, credentials, tools, and model execution in Rust.

The framework-independent `@tanstack/ai-client` remains an alternative if a
custom React hook is preferable. TanStack AI owns client chat state, not catalog
loading, application use cases, or provider authentication.

Sources:

- [TanStack AI connection adapters](https://tanstack.com/ai/latest/docs/chat/connection-adapters)
- [TanStack AI React hook source](https://github.com/TanStack/ai/blob/main/packages/ai-react/src/use-chat.ts)
- [TanStack AI client adapter interface](https://github.com/TanStack/ai/blob/main/packages/ai-client/src/connection-adapters.ts)
