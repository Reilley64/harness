# Tool calls

Status: implemented. Rust owns the model/tool loop, durable execution records,
and steering. React/Ink renders ACP updates; it does not execute tools or run a
second model loop.

## Tools and adapter ownership

| Model name | Domain invocation | Infrastructure port module |
| --- | --- | --- |
| `read` | `ReadFile` | `read_file` |
| `bash` | `CreateProcess` | `create_process` |
| `edit` | `UpdateFile` | `update_file` |
| `write` | `CreateFile` | `create_file` |
| `ls` | `ReadDirectory` | `read_directory` |
| `find` | `ReadPaths` | `read_paths` |
| `grep` | `ReadTextMatches` | `read_text_matches` |
| `ast_search` | `ReadCodeMatches` | `read_code_matches` |
| `web_search` | `ReadWebSearch` | `read_web_search` |
| `publish_work` | `PublishWork` | `read_work_proposal` + application `create_work` |
| `mcp_configuration` | `McpConfiguration` | `update_mcp_configuration` |
| `mcp_call` | `McpCall` | `create_mcp_call` |

`domain::Tool` identifies the fixed dispatch set (dynamic MCP identities are typed invocation fields); `ToolInvocation` carries typed
arguments. Each adapter owns its strict camelCase `Arguments` representation and
JSON schema. Unknown fields, malformed JSON, disabled tools, and unavailable tools
produce invalid-call results without executing anything.

Tools are enabled by default. `TOOLS__ENABLED=false` disables advertisement and
execution for text-only operation. AST search is advertised only when
`ast-grep --version` reports exactly `ast-grep 0.51.0`. Trusted configured stdio
MCP servers use the same durable tool loop; see [MCP connections](mcp-connections.md)
for native configuration, protected trust, discovery and shared cleanup. HTTP
execution is not yet available. There are no plugins, managed child dispatch,
browser automation, PTYs, persistent shells or managed background jobs. Trusted exact-version
[approved handoff](approved-work.md) can create a separate conversational CoS;
model tools cannot grant human approval.

## Permissions and workspace

Enabled native tools and already-trusted MCP invocations run automatically,
without per-call approvals or permission cards, with
the OS user's permissions. The workspace is a working directory, not a sandbox.
Absolute paths and readable files outside it are accessible, including credentials.
Harness does not proactively inject OAuth tokens into tool arguments, prompts,
or process environments. Explicit file/command output can nevertheless contain
secrets the user can read.

The `bin/harness` launcher captures Mise's invocation directory, explicit
`INIT_CWD`, or its shell cwd before delegating to the Mise TUI task. The TUI sends
that launch directory through ACP; Rust build and launch commands are not exposed
through Bun package scripts. New conversations canonicalize and persist it; resumed conversations
retain their saved workspace. Legacy records bind the supplied resume workspace on
their next mutation.

## Authoritative loop and continuation

1. Lock the conversation and persist the user message/in-progress assistant.
2. Ask the provider for one model step, with ordinary Responses function tools,
   `parallel_tool_calls:false`, `store:false`, and encrypted reasoning included.
3. Require a terminal successful provider event before accepting calls. An empty
   terminal output array does not erase completed streamed output items. A step
   with neither text nor calls fails explicitly. Persist the full bounded
   continuation and requested calls before execution.
4. Validate arguments; persist Running before each enabled valid invocation.
   Execute sequentially, persist its terminal result, then publish its ACP update.
5. Pair every recorded provider call with its honest result before further
   inference. Invalid calls and ordinary failures are results the model can handle.
   Unknown cleanup/outcomes stop the turn.
6. Persist final or partial assistant text and the terminal message status.

The adapter retains provider output items, call IDs, and opaque reasoning for
stateless continuation. Opaque data never enters ACP snapshots. When changing
provider or model, private reasoning and provider item/call identities are omitted;
ordinary text and reconstructed requests with honest results remain. The new
provider uses its independently stored credential. OpenRouter requires the
requested parameters and disables endpoint fallback and gateway plugins; a
reported incompatible model is an error before tools, never a replacement worker. Reused provider call IDs are rejected before new execution.
HTTP/provider/protocol failures are surfaced; there is no hidden provider fallback.
Use the explicit text-only setting for a model that does not support function tools.

Records are schema version 6, retaining v5 per-message/run provider provenance, optional
resolved-model/serving-provider metadata, agent/run associations and immutable
instruction snapshots. Versions 1 through 5 remain readable without rewriting
until mutation/recovery; their single stored provider supplies missing legacy
provider provenance, and historical instruction provenance is never invented.
See [role guidance](role-guidance.md).
Read/create/update conversation adapters and preference adapters each own their
serialized `Record` types. Shared filesystem helpers operate on bytes, not DTOs.

## Steering and cancellation

Enter during a turn calls `_harness/conversation/steer` with a UUID and instruction.
Rust deduplicates accepted IDs and acknowledges only after persistence.
The original user message is not copied into another turn.

During inference, accepted steering cancels the in-flight provider step and
continues the same turn with the instruction. Partial prose remains visible and
saved but is not replayed as completed model output. During a tool, the current
operation finishes; remaining calls in that batch are recorded Skipped before
continuing with the new instruction. Rapid instructions retain their order.

Explicit Stop cancels the current turn, including subprocess cleanup, without
initiating another request. Escape only closes or back-navigates. Failed/unaccepted instructions remain drafts. Unsaved draft text
is not cleared by completion. Neither React nor the transport automatically
replays a failed request.

## Tool behavior

- `publish_work`: capture a real proposal file, immutable source document/version
  and explicit publication intent through the application use case. Amendments
  invalidate affected authority; this tool cannot approve or launch implementation.
  See [approved work](approved-work.md) for schema, idempotency and bounds.
- `read`: regular UTF-8 files only, 1-based offset, default 200/max 2,000 lines.
  The input file is bounded at 1 MiB. No image or binary decoding.
- `bash`: noninteractive `/bin/bash --noprofile --norc -c`, no stdin or shared
  shell state. Concurrent stdout/stderr draining is bounded. Timeout/cancellation
  sends TERM to the process group, then KILL after up to two seconds, and reaps
  the leader. Unconfirmed cleanup is Unknown. Normal completion also cleans up
  remaining group descendants. Commands can deliberately escape a process group;
  this is not containment.
- `edit`: every old string must occur exactly once in the original file, including
  overlapping occurrences. All replacements must be non-overlapping. The whole
  batch commits atomically, with a bounded unified diff.
- `write`: atomically replaces a complete UTF-8 file, creating parent directories.
  File contents are bounded at 1 MiB.
- Both mutations serialize by canonical path across Harness processes, reject
  symlink/nonregular destinations, preserve existing permissions, compare original
  contents again before commit, and sync the file and parent. New files use
  ordinary file-creation permissions restricted by the process umask. External-editor races cannot be eliminated
  by an advisory Harness lock; no rollback or exactly-once guarantee is made.
- `ls`: sorted direct entries, including hidden entries and symlink labels.
- `find`: native glob matching over an ignore-aware walk, without following links.
- `grep`: native Rust regex/literal search, optional glob, case folding and up to
  five context lines. Ignore files are respected; binary/non-UTF-8/over-1-MiB files
  are skipped. Zero matches is success, not failure.
- `ast_search`: read-only structural ast-grep matching, not type-aware resolution.
  Its schema lists supported language IDs; runtime names are case-insensitive.
  Harness supplies its own minimal configuration under app data and disables custom
  language loading. No project parser libraries are loaded.
- `web_search`: direct native DuckDuckGo HTML search, first-page organic titles,
  resolved HTTP(S) URLs, and snippets. Ads are excluded. There is no result-page
  fetch, cache, hidden provider fallback, or challenge bypass. HTTP failures,
  throttling/challenges and unrecognized layouts are explicit failures; recognized
  empty searches succeed.

## Panels and ACP

Standard ACP tool-call and tool-call-update notifications carry a bounded
`_meta["harness/tool"]` summary. Conversation snapshots include the same tool
summaries and accepted steering instructions; opaque provider state is excluded.

Whole panels update in place: grey Pending/Running, muted green Succeeded, and
muted red Failed/Invalid/Unknown. Cancelled/Skipped/Interrupted remain grey and
labelled. A successful tool stays green even if its parent turn is cancelled.
Bold headers show paths, requested line ranges, timeouts, and other relevant
parameters instead of successful-status labels or elapsed duration. Non-success
statuses remain labelled without relying on colour. Read/write code has syntax
highlighting for supported file types; edits have unified-diff colouring. Unknown
languages remain plain text. Bounded output and artifact/truncation notices are
displayed. Ctrl+O expands/collapses stored output. Mouse wheel, PgUp/PgDn, and Shift+Up/Down scroll even during a turn;
new output preserves the viewed history until scrolling back to the bottom.
Escape preserves the anchor while navigating. Shift+drag uses native terminal text selection while mouse
reporting is enabled. The prompt remains full-sized and editable.

## Bounds and artifacts

- 64 model steps per turn, 16 calls per step, 64 valid executions per turn.
  The final model step is tool-free and reserved for an honest results summary;
  execution-budget exhaustion also triggers this summary. Such turns persist
  with status `limited` and ACP stop reason `max_turn_requests`, not a generic
  failure. New steering is rejected while the budget summary is being generated.
- 64 KiB arguments; 4 MiB SSE events; 1 MiB continuation per step/4 MiB per turn.
- 1 MiB each for user text, total assistant text, and accepted steering text.
- 64 steering instructions; bounded 32-entry control channel.
- 20-minute turn deadline; bash default 120 seconds/max 600 seconds.
- Searches: default/max 100 matches; native walks max 100,000 entries/10 seconds.
- AST search: 10 seconds; DDG: 15 seconds, 2 MiB response, max 10 results.
- Captured process stdout and stderr: up to 1 MiB each, then drain/discard.
- Tool preview text and its optional artifact reference together occupy at most
  32 KiB as JSON string/null values, including escaping and the truncation note.
  Head/tail cuts preserve UTF-8 boundaries. Escaped controls can require an artifact
  even when the original output is below 32 KiB; retained artifact bytes are not
  JSON-escaped.
- Large outputs: up to 1 MiB combined UTF-8 artifact per call under
  `tool-outputs/<conversation UUID>/<call UUID>.txt`, with a 128 MiB conversation
  budget. A budget-limited artifact is explicitly unavailable; the preview remains.
- Conversation metadata: 16 MiB, with terminal-result capacity reserved before
  execution for all outstanding calls simultaneously. The reserve uses the enforced
  encoded preview/artifact bound plus worst-case terminal status, elapsed duration,
  truncation flag, and finishing message/revision/timestamp metadata; existing call
  arguments and history are included in the current encoded record. A capacity
  rejection happens at the checkpoint before effects, not after a result is known.
  Storage and artifacts use private app-data directories/files.

## Verification

Temporary external probes cover all nine native adapters, atomic edit validation,
outside-workspace paths, UTF-8, regex/AST zero matches, process failures,
timeout/cancellation/descendant cleanup, output truncation, fixture Responses
continuation with opaque reasoning, strict argument failures, ACP/TS SDK live
updates and resume, inference/tool steering, deduplication, crash recovery and
legacy migration. Ink probes cover status panels at 50/80/120 columns and
editable-prompt/steering races.

A live DDG query succeeded during verification. This is not a reliability
benchmark. Function calling is verified against the local Responses/OAuth fixture and a
live ChatGPT-connected `gpt-6-luna` smoke test: all nine tools succeeded and the
turn returned a text summary. Other account/model combinations may differ.
