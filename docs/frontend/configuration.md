# Configuring and inspecting work

## MCP configuration

`/mcp` opens native shared configuration without starting a conversation or
inference. The MCP hook owns asynchronous ACP operations/cancellation, the
service validates transport responses, and the panel renders effective source,
trust status, nonsecret definitions, tool schemas and optional registry metadata.
Connect has a paged exact-definition review and explicit confirmation; Escape
cancels review without activation. Custom saves, manual files and explicit reload
share the backend/schema with the agent configuration tool. The command draft
and conversation stay intact while the panel is open. Provisioned references,
not secrets, enter configuration. Stdio and Streamable HTTP use the same backend
configuration and tool pipeline. See [MCP connections](../mcp-connections.md) for
commands, keys, errors and bounds, and [HTTP and OAuth](../mcp-http-auth.md) for the
supported transport/client profiles and private credential lifecycle.

For OAuth Connect, the service sends a fresh `authentication_id` and matches
`_harness/mcp/authorization` notifications against that ID, workspace and exact
definition revision. The hook also fences presentation against its active,
uncancelled operation. Authorization links stay in a protected native pane,
hidden by default: `o` explicitly opens the link, `l` reveals/hides it, and
Left/Right pages a revealed link. Resize resets paging. Escape cancels the pending
attempt and invalidates its presentation, without claiming to undo a committed
save or credential update. Closing the panel does not revoke shared tool access.
Protected actions require at least 50 columns × 18 rows; below that size they are
blocked while Escape remains available. MCP modals consume bracketed paste
without activating shortcuts.

Cancellation keeps the underlying MCP response wait so admitted backend commits
can finish; pending MCP requests join the bounded 60-second shutdown drain.
Response loss requires read/reload reconciliation, not silent login or replay.
External clients should follow the exact
[ACP MCP challenge and cancellation contract](../acp-presentation.md#mcp-oauth-challenge-correlation).

## MCP controls

- `/mcp`: inspect effective server names, source, status and revision.
- `/mcp reload`: validate manual edits and replace the effective snapshot.
- `/mcp save <nonsecret JSON definition>`: add/replace one app definition using
  the last observed configuration revision. Open `/mcp` first if no revision has
  been read. Close the panel with Escape before entering another command.
- `/mcp remove <name>`: remove an app definition, not a repository override.
- `/mcp registry <search>`: optionally read registry metadata. `v` opens paged
  metadata; it is untrusted data, not installation/launch authority or a complete
  login recipe. Package names, required/secret flags and documentation links are
  hints for a definition the user/agent prepares, never automatically executed.

In the panel, Up/Down selects a server, `c` reviews Connect, `d` revokes local
trust and retires the connection, `t` discovers tools, and `r` reloads. Exact
nonsecret definitions are reviewed in ASCII-escaped pages with Left/Right;
approval is enabled on the last page. Resize returns review to the first page.
Enter explicitly approves that exact revision. Escape cancels review without
activation, or closes the panel/cancels the wait. An already completed save or
trust commit is not rolled back by dismissing a pending operation. Pre-cancelled
operations and cancelled configuration-mutex waiters start no persistence or
activation. Save/remove and protected trust updates check cancellation again at
write admission; after admission, the commit and snapshot completion are awaited
rather than abandoned or rolled back. Connect also checks cancellation before
process launch, so a saved trust decision need not imply successful discovery.
Read/reload to reconcile a completed commit whose discovery or response was lost.
Conversation state and the editable command draft are not submitted as chat by these controls.

Connect records local trust and initializes/discovers the configured server.
Configured OAuth first requires successful explicit browser authorization; saving
trust is neither successful authentication nor permission to send a tool mutation. Subsequent
`trusted` status describes consent/eligibility, **not a health or liveness
claim**; discovery/execution reports current failures. Closing the panel does not
disconnect tools used by other conversations. A failed executable/runtime or
missing provisioned input requires correcting configuration or following the
server's documentation, not silently installing a package or restarting Setup.

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
[provider connections](../provider-connections.md).

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

## Native Decider controls

`/decider` opens the separate native release/alias picker; `/decision <intent>`
opens its scrollable durable inspector. `s` selects once for an actual prepared
CoS intent, `x` then `y` explicitly Stops, and Escape navigates without cancelling.
`o` opens the actual CoS for discussion. Configuration blocks open the existing
model settings without choosing a replacement. The picker never changes worker
selection or exposes native thinking. These controls use real ACP/application
paths and preserve per-conversation drafts/viewport; no child is launched.
See [native Decider](../native-decider.md) for policy, input limits, provenance,
pending/restart behavior and the live/PTY evidence boundary.

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
replacement. See [model allowlist](../model-allowlist.md) for policy and persistence.

## Instruction provenance

`/guidance` opens the real ACP-backed [guidance inspection](../role-guidance.md),
including selected/latest-run/available snapshots, exact source/version/upstream,
required blocks and explicit overrides. Scoped/reference/phase commands inspect
candidates; `/guidance adopt` records explicit adoption for future work. Bundled unimplemented roles remain assets only; Research execution is documented
by [managed dispatch](../managed-dispatch.md). The approved-work CoS is a separately
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
ownership. See [approved work](../approved-work.md) for authority, repeat clicks,
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
changes. See [repository Setup](../repository-setup.md) for keys, scope, schema,
bootstrap instruction handling and the trusted-client/OS-user boundary.
