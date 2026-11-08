# Frontend

The frontend uses React and Ink, with Bun as both the package manager and the
TypeScript/TSX runtime. `bun.lock` is the committed lockfile. There is no npm
lockfile, separate JavaScript build, browser DOM, or `tsx` launcher.

## Atomic design

```text
presentation/tui/src/
├── components/
│   ├── atoms/
│   │   ├── panel.tsx
│   │   └── terminal-text.tsx
│   ├── molecules/
│   │   ├── message-line.tsx
│   │   ├── model-option.tsx
│   │   ├── model-pricing.tsx
│   │   ├── model-search.tsx
│   │   ├── prompt-box.tsx
│   │   ├── provider-option.tsx
│   │   ├── startup-wordmark.tsx
│   │   └── working-indicator.tsx
│   ├── organisms/
│   │   ├── model-picker.tsx
│   │   ├── provider-connection-status.tsx
│   │   ├── provider-picker.tsx
│   │   └── transcript.tsx
│   └── templates/
│       └── harness-layout.tsx
├── hooks/
│   ├── use-agent-connection.ts
│   ├── use-conversation.ts
│   ├── use-create-provider-connection.ts
│   ├── use-model-picker.ts
│   └── use-provider-picker.ts
├── screens/
│   └── harness-screen.tsx
├── services/
│   ├── acp.ts
│   ├── authorization.ts
│   ├── conversations.ts
│   ├── models.ts
│   └── transcript.ts
├── types/
│   ├── conversation.ts
│   ├── model.ts
│   ├── provider-connection.ts
│   ├── provider.ts
│   └── transcript.ts
├── commands.ts
├── index.tsx
└── theme.ts
```

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

## Provider connections

Selecting ChatGPT or OpenRouter in the provider picker calls ACP `authenticate`
with its separately advertised method. OpenRouter authorizes a separately billed
API key via backend PKCE; connecting does not make a paid inference probe. The connection
hook presents starting, browser-authorization, failed, and connected states.
Escape cancels a pending flow; `o` explicitly reopens the sign-in URL. The prompt
is unfocused while authorization is pending, but global quit hotkeys still work.

The transport correlates browser challenges to requests using a fresh opaque
authentication ID, ignores stale or unrelated challenges, validates responses,
and sends ACP cancellation when the hook aborts. Authorization URLs must use HTTPS
or loopback HTTP and may not contain URL credentials. Infrastructure opens the
browser initially; the frontend's `open` dependency is only used for manual reopen.

Tokens, OAuth exchanges, ID-token verification, and credential persistence stay in
Rust. Successful connection metadata contains only opaque connection/provider IDs,
not account emails or subjects. Sign-in URLs are hidden by default; `o` opens the
browser, and `l` explicitly toggles the link for manual sign-in. Raw agent stderr
is drained and discarded rather than surfaced as a diagnostic. See
[provider connections](provider-connections.md).

## Model picker

`/model` opens an inline picker inspired by
[Prime Agent's model picker](https://github.com/PrimeIntellect-ai/prime-agent/tree/main/crates/pa-tui/src/model_picker):
bordered search, selectable model rows with inline effort squares, and a separate
input/cached-input/output pricing block.

- Search matches case-insensitive words across model name, ID, and provider.
  Typed and pasted text appends at the end; Backspace removes a code point and
  Ctrl+U clears it. Left/right are reserved for effort, not search-cursor movement.
- Up/down move through the filtered list; the viewport follows the highlight.
  Left/right move through that model's supported effort levels and clamp at the
  ends. Models without capability metadata have no effort control.
- Medium is preferred when supported; otherwise the lowest supported level is
  used. Draft efforts are remembered per model within an open picker.
- Enter commits the highlighted model and effort. Escape discards draft changes.
  Reopening restores the committed model/effort when still available; unsupported
  saved effort falls back to a supported level.
- Ctrl+R reloads the catalog, subject to the backend cache TTL. Loading, failure,
  empty catalog, and no search matches have explicit states.
- The bottom-right footer shows provider/model ID and raw effort ID, for example
  `openai/gpt-6.1-sol · extra_high`; command hints stay on the left. Committing a
  picker selection saves it through Rust for future fresh sessions, without
  creating a conversation. Escape discards drafts without saving.
- Pricing is explicitly catalog USD per million tokens, not subscription billing.
  Missing and free prices are distinct. Unknown, duplicate, or malformed model
  metadata is rejected by the transport parser.
- The bottom-anchored picker replaces the main prompt and footer while open.
  Its search field rises only enough to fit the results, pricing, and one-line
  controls below it. Blank lines separate pricing from the list and controls.
  At most eight model rows are shown, reduced for the minimum 50×18 terminal.
  The controls show navigation, effort, selection, and Escape; typing to search
  and Ctrl+R retry remain active without visible hints. Closing or confirming the
  picker restores the normal prompt and footer.
- Effort names reserve ten columns and squares reserve four, keeping the names,
  squares, and arrows aligned while changing effort or moving between models.
- Pending requests are aborted on dismissal, disconnect, reload, or unmount.

Data comes from `_harness/models` through the existing application `read_models`
use case. Its only runtime model source is the Harness-owned JSON at `MODELS__URL`,
with one bounded memory/disk cache and the existing remote-first disk fallback.
There is no secondary OpenRouter catalog fetch, merge or cache. Ordinary workers
require a positive `chat` capability; empty, missing, legacy boolean and
`decisions`-only metadata never authorize chat. Hosted routers remain blocked;
missing effort metadata adds no thinking controls. Explicit provider/model switches apply on the next prompt,
preserving history and usage while backend translation excludes incompatible
private continuation. The frontend never fetches the HTTP catalog directly.

Temporary external probes cover the real Rust/SDK path, catalog metadata,
fallback/retry, parser rejection, search, effort changes and limits, confirmation
and cancellation, scrolling, terminal resize, and clean shutdown. No permanent
test files or test-only dependencies were added.

## Conversations

`useConversation` owns optimistic streaming display, explicit cancellation,
error reconciliation, and the last authoritative snapshot. Rust owns the turn/tool
loop and steering; the hook sends instructions rather than cancelling/resending.
An instruction-adoption acknowledgement updates only its same conversation target
at a non-older revision; a delayed reply cannot erase a newer completed turn or
regress the map's run state. A stale initial Griller-creation acknowledgement is
rejected before adopting instructions, without replacing newer history; inspect
again to adopt against the current conversation. Durable commits are not rolled
back by these presentation guards. The hook also owns Griller lookup through
conversation load and model restoration, aborting/invalidation on replacement,
navigation, send and unmount. Interruption acknowledgements require the same
selected actor, current navigation ownership and a non-older revision; a durable
acknowledgement cannot overwrite newer history or another actor's context/model.
The first send creates a session; opening a picker or choosing a model does not.
After failure the hook reloads saved history so it does not mistake optimistic
text for persisted messages or automatically resend a failed turn.

The transcript renders message text and partial statuses in a bounded native Ink
viewport. Each assistant text section has top/bottom padding, including prose
between tool calls and the final summary. Message sections extend to both terminal
edges;
there are no user/assistant headings, conversation ID banner, paging hints, or
model-selection notices. Per-reply provenance stays in durable metadata. Its Pi-inspired styling gives user
messages blue-tinted sections, keeps assistant replies on the terminal background,
and uses amber/red bands for interrupted/cancelled/incomplete/refused and failed
replies. Empty pending assistant messages do not add a duplicate thinking panel;
the prompt accent is violet.
Native Ink background/padding props paint each visible row, including clipped
sections during paging. `services/transcript.ts` formats stable, sanitised rows;
`MessageLine` renders them, and `theme.ts` centralizes the palette. Panel foreground
and background colours are paired; assistant body text uses the terminal default.
Labels/status text remain meaningful with colours disabled. `wrap-ansi` measures wrapped lines for
PgUp/PgDn paging; display primitives still sanitize control characters. The
wordmark depends on actual visible messages, not commands or picker activity.
During a reply the prompt stays focused and editable. Enter submits an acknowledged
instruction to Rust's control channel. It continues the same logical turn, without
duplicating the user message or replaying unfinished prose. Running tools finish
before remaining batch calls are skipped. Escape closes/back-navigates only. Rejected submissions
retain their exact submitted whitespace. The latest rejection restores directly to
an empty editor when no rejected drafts are already queued. Otherwise it enters a
session-local rejected-draft queue without replacing newer text, even if newer
submissions have already been sent. A visible notice counts queued drafts; clear
the editor, then press Ctrl+R to recover the oldest known rejection in submission
order. Recovery only fills an empty editor and never sends automatically; Enter
sends the recovered text normally. Out-of-order failures are sorted in the queue,
not concatenated, and accepted sends do not clear it. This also covers transport
errors, cancellation before acceptance, missing-model and oversized rejections.
Queued drafts are held in memory until recovered or the screen exits, not saved to
conversation history. Late send results do not update draft state after unmount.
Completion does not erase text being edited. Slash commands other than quit
require explicit Stop first, except navigation and Stop itself.

Tool notifications update stable call IDs in place. Padded grey panels become
muted green on success or muted red on failure/Unknown; cancelled/skipped calls
stay grey. Parent-turn cancellation does not recolour successful tools. Tool
headings are bold and show paths, requested line ranges, timeouts, and other
relevant parameters instead of success labels or elapsed durations. Non-success
states remain explicit. Read/write code uses extension-based syntax colors; edits
use unified-diff coloring. Successful writes include a preview of the written
content. Highlighting is cached per immutable call, bounded, and falls back to
plain text for unknown languages or unsupported output. It never embeds terminal
escape sequences from tool output or changes panel backgrounds.
Ctrl+O expands/collapses output; resume reconstructs the same panels from snapshots.
Mouse wheel, PgUp/PgDn, and Shift+Up/Down scroll during both generation and idle.
The viewport fills its measured available height, bottom-aligns short transcripts,
and preserves its anchor while output streams; reaching the bottom resumes
following new output. A screen-owned viewport hook retains independent anchors
and tool-expansion state by durable conversation identity for the current UI
session. Switching Griller/CoS, opening pickers or visiting an empty conversation
cannot reuse another actor's numeric anchor or erase the previous reading position.
This is in-session state, not durable scroll restoration after process restart. Escape preserves the anchor while navigating. Native terminal selection uses
Shift+drag while mouse reporting is enabled. Mouse modes are restored on unmount.
Unchanged immutable message objects retain cached row layouts; each message keeps
only its current width/expansion layout in a weak cache. Visible rows are memoized,
wheel events move one row, and Ink renders at up to 60 FPS. See
[tool calls](tool-calls.md) for permissions, continuation, limits, and verification.
The prompt reserves its content row even when empty/unfocused and cannot shrink
under layout pressure. Multiline drafts expand within a terminal-sized viewport;
only rendering is clipped, and submission sends the entire draft unchanged.
Oversized submissions fail visibly and preserve the draft. A starburst
indicator occupies a separate full-width row outside the scrolling viewport, with
the same horizontal and vertical section padding as chat. It remains immediately
above the prompt, followed by unbracketed elapsed time. The prompt border and
active thinking text share a smooth three-minute rainbow cycle through a subdued
Pi-dark-inspired palette; one 250 ms clock updates only its subscribers, not the
screen or transcript. Picker borders remain static. Theme foregrounds, user/tool
backgrounds and syntax colors follow Pi's built-in dark palette rather than ANSI
magenta; this is not Pi's adaptive system-theme implementation. It switches to grey
`Worked for …` when the send/steering chain finishes, including
cancellation or failure. Its monotonic elapsed time covers session preparation,
generation, and durable finalization, not provider-reported reasoning time. The
spinner timer stops when idle or unmounted; the shared color timer runs while an
active prompt or thinking indicator is visible and stops after its last subscriber
unmounts. Startup takes raw input ownership before rendering and enables Kitty
keyboard reporting without a capability query, avoiding echoed probe replies;
unsupported terminals retain Alt+Enter/Ctrl+J, and Ink restores reporting on exit. Tab or right arrow accepts the
prompt's ghost command completion; normal input navigation remains when there is
no completion. Terminal selection remains terminal-controlled: Ink cannot
selectively make only the background ASCII wordmark unselectable.

The footer has two rows: the bound workspace and its current Git branch above the
controls, and cumulative provider-reported input/output tokens above the model
name. The branch appears as `(main)` beside the path, or `(detached <short-id>)`
for detached HEAD. Read-only, bounded Git queries refresh about every two seconds
against the bound workspace (including resumed workspaces and linked worktrees),
not the Harness checkout. Missing Git and non-repositories show no branch label;
queries are cancelled on workspace changes/unmount. Long labels truncate within
the existing footer row. `ctx` is the latest
reported request's input plus output tokens divided by the selected model's
optional catalog `context_window`; it is not cumulative token billing or account
quota. It excludes unsent drafts and changes since that reported request. Unknown
usage/capacity displays `—`; missing generation reports and historical gaps mark
known totals with `≥`. Reported superseded or cancelled generations still count;
steering does not itself make complete reports partial.
Per-step reports update live and are restored from durable history. No local
character-to-token guesses or fabricated model limits are used.

The entry point accepts `--resume <conversation-id>` and loads it before rendering.
Invalid/missing/nonexistent identifiers fail visibly. The saved model/effort seeds
the picker without requiring the catalog. No arguments detect missing applicable Setup outputs and open the saved Setup
actor when incomplete, otherwise the saved repository Griller; an absent actor
renders an empty view without creating history.
When that repository actor does not exist, startup restores the last model/effort through
`_harness/model-selection/read`, without credentials, catalog access, or history.
Explicit resume takes precedence and does not overwrite this default just by
reading a conversation. A missing preference is normal; an invalid one produces
a nonfatal warning and can be replaced by choosing a model. Picker commits are
serialized through `_harness/model-selection/update`; exit waits for these saves.
`/open <conversation-id>` opens an existing saved conversation while idle;
`F3` or `/griller` returns to the repository Griller. No general conversation
picker or background conversation switching is implemented here.

The ACP service translates standard sessions/prompts/updates/cancellation, parses
safe conversation metadata, and never receives credentials. On exit it cancels
active replies and waits for their durable finalization before closing the
subprocess. See [conversations](conversations.md).

## Model permission settings

The existing `/model` picker labels locally blocked entries and shows the
authentication, allowlist or worker-capability reason. Ctrl+A opens its
application-wide positive-pattern editor, even when the catalog fails to load.
Enter saves the complete list, Ctrl+U clears it and Escape discards the draft,
including while the settings read is loading. Reads are aborted on dismissal,
reload, disconnect and unmount, and late results are ignored. Saving is a separate
commit state; shutdown drains writes and does not claim rollback.
An empty list permits no inference; overlapping broad positives still permit
covered identities. No settings dismissal retries blocked work or selects a
replacement. See [model allowlist](model-allowlist.md) for policy and persistence.

## One-agent context and navigation

The native map contains the actual repository Griller identity returned by Rust,
not a static role roster. The node persists across human answers, model changes,
stopped/completed runs and reloads. It shows the latest run status (or live local
work status) and an explicitly unknown creator; no creation arrow or descendant
is fabricated. While local work is pending, unavailable live run details are
explicitly marked pending; saved run identity/model/status are labelled historical
rather than presented as the active run. Legacy pilot conversations remain
unassociated. Multi-agent growth, pan/collapse and execution subtrees belong to
later slices.

Plain Left from the empty focused editor invalidates any pending conversation
open before showing the map; whitespace, modifiers,
paste and picker input retain their editing meaning. Function-key-shaped bytes
inside bracketed paste never execute navigation, including fragmented markers
and large paste chunks. Paste tracking survives picker activation changes.
Ink retains its 20 ms ambiguity timeout for incomplete escape prefixes; recognized
bracketed paste is not timeout-flushed. Synchronous event-batch
editing state prevents coalesced space-plus-Left from navigating accidentally.
F2 toggles chat/map; F3 returns to chat in the one-Griller context. Enter opens
the sole selected node; Tab/arrows retain that selection. A single mouse click
selects it; a same-node double-click within 400 ms opens it. Ink-measured hit bounds
handle split SGR reports and invalidate pending double-clicks on geometry changes.

Titles are exactly `harness - Griller`, `harness - Setup` and `harness - Agent map`; an explicitly
loaded unassociated pilot uses `harness - Conversation`. Only static context labels
enter title escapes. Background updates cannot change selection/title. Title,
mouse and raw/alternate-screen modes pair with cleanup. The transcript stays
mounted behind the map to retain its anchor and unsent drafts are untouched.

Escape closes a picker, Stop confirmation or authentication flow, or navigates
between conversation and map; it never stops a run. Use `/stop` or Ctrl+X on the
map for leaf Stop confirmation. Y/Enter confirms; Escape/N closes without
cancelling. Confirmation explains that completed effects remain and uncertainty
is not rollback. Ctrl+X inside the editor cuts its current logical line, not the
run. ACP cancellation remains available without adding a human approval gate
to backend/tool authority.

Interrupted work displays Resume-required state. `/resume` explicitly continues
the same run. For uncertain effects, `/resume inspect` acknowledges the interruption
without generation, retaining Unknown results and permitting a new inspection
instruction. Reopening, map navigation or ordinary prompts cannot start recovered
work. See [conversations](conversations.md).

External stock-launcher PTY probes cover 80×24/50×18, exact title bytes, actual
backend kill/restart/Resume, empty versus whitespace/modified Left, Unicode
multiline paste/draft retention, F2/F3, split SGR clicks, Escape during inference,
confirmed Stop and normal terminal-mode restoration. These byte-report
observations are not a human terminal-emulator visual signoff.

## Instruction provenance

`/guidance` opens the real ACP-backed [guidance inspection](role-guidance.md),
including selected/latest-run/available snapshots, exact source/version/upstream,
required blocks and explicit overrides. Scoped/reference/phase commands inspect
candidates; `/guidance adopt` records explicit adoption for future work. Bundled later roles remain assets only. The approved-work CoS is a separately
created actual conversation with recorded creator and coordination instructions;
inspection alone never creates it. Inspection
preserves the mounted chat, drafts, titles and Escape-back behavior. It does not
start inference; active runs continue with their captured snapshot.

## Approved work and CoS

`/work <work-id>` opens a scrollable exact-source/version/scope/criteria proposal
inspector. `a` then `y` records trusted approval and hands off to a separate durable
CoS through real ACP/application operations; `n`/Escape declines/dismisses. No
chat/prose/model tool is approval. `o` opens the delivered CoS without inference.
The title is `harness - Chief of Staff` for that conversation. `F3` returns to
Griller, and per-conversation draft buckets retain unsubmitted and rejected text.
Stale read/open/approval results cannot take over a newer target/view. Admitted
writes are drained on shutdown, not falsely rolled back by dismissal. Both roles
retain ordinary selected-leaf Stop and explicit Resume; creator is not Stop
ownership. See [approved work](approved-work.md) for authority, repeat clicks,
publication conventions, missing guidance recovery and unavailable later execution.
The current map describes the selected real agent, not a simulated full fleet.
## Repository Setup

Repository open selects the actual durable Setup actor when applicable output files
are absent; complete repositories bypass it. `/setup` inspects presence and exact
stored before/after proposals, with explicit per-proposal confirmation/cancellation.
`/setup chat` and `/griller` navigate independent durable actors without inference.
F3 opens the actual Griller from Setup/map through the hook-owned navigation
lifecycle, preserving per-actor drafts and scroll. Inspector and approval target
labels use lossless ASCII JSON strings; the underlying approved path is unchanged.
The current-role map remains a leaf, not the later multi-agent map product. Draft
inspection is cancellable; admitted writes drain without late navigation/title
changes. See [repository Setup](repository-setup.md) for keys, scope, schema,
bootstrap instruction handling and the trusted-client/OS-user boundary.

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
