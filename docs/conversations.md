# Conversations

## Scope and terms

A conversation has a stable UUID, a provider-connection reference, a current
model/effort, ordered messages, timestamps, and a revision. A message has its own
stable UUID, user/assistant role, text, status, and model/effort provenance. Message
roles are separate from agent roles such as griller or researcher.

Messages are UTF-8 text with durable tool steps and accepted steering instructions.
Ten automatic tools are supported, including versioned proposal capture. A separate
approved-work CoS uses the same loop; child orchestration, attachments and a
conversation picker remain unavailable. See [approved work](approved-work.md). OAuth tokens stay in Rust and are not proactively
injected into messages or ACP; explicit tool output may contain readable secrets.
See [tool calls](tool-calls.md) for the execution and permissions contract.

## Workflow

1. Startup reads this repository's Griller when one exists, without inference.
   Otherwise it renders an empty view and creates no conversation file.
2. The user signs in with `/auth` and chooses a model with `/model`.
3. On the first idea, Ink requests `_harness/agent/start`, backed by
   `create_agent`. It validates model/provider/effort, resolves the saved connection,
   and atomically creates the repository Griller or opens its existing conversation.
   Standard ACP `session/new` remains the unassociated pilot conversation interface.
4. ACP `session/prompt` invokes `create_message`. A per-conversation file lock
   rejects overlapping replies, including from another process. The user message
   and an empty in-progress assistant are saved before contacting the provider.
5. `CreateModelStepFactory` selects a provider-bound `CreateModelStep` function.
   `PresentTurnEvent` forwards stable-message-ID text and tool updates to ACP.
   Application buffers text and persists tool checkpoints before continuation.
6. Completion, cancellation, and failure finalize the assistant with its partial
   or complete content. Failed generation returns an error only after saving the
   failed reply; Ink reloads the saved record to reconcile its temporary view.

Changing the picker selection saves the last-selected model/effort preference,
not a conversation. The next
message validates and saves the new model/effort and preserves older replies'
provenance. A subsequent prompt can explicitly switch to the other connected,
allowed compatible provider. Its own saved connection is resolved; earlier messages
and runs retain their original provider/model/effort and reported usage. A switch
never authorizes replay of an uncertain tool effect.

Current application-wide model permission is revalidated before every inference
step and again at provider HTTP admission after credential resolution. Revocation
preserves started effects, results and usage but blocks the next disallowed
request without a replacement. See [model allowlist](model-allowlist.md).

## Provider requests

OpenAI inference uses `POST https://api.openai.com/v1/responses` with the subscription
Bearer token, `store: false`, `stream: true`, and text history. Effort maps to
`low`, `medium`, `high`, or provider spelling `xhigh`.

OpenRouter uses its own key with `POST https://openrouter.ai/api/v1/responses`
through the same Responses HTTP/SSE decoder and existing application tool loop.
It sends the exact selected nested model ID, `store:false`, `stream:true`,
`provider:{allow_fallbacks:false,require_parameters:true}` and an empty plugin
list. No model fallback list, hosted advisor, paid connect probe or retry is added.
A returned OpenRouter model must match the requested ID or its exact catalog
canonical slug before accepting tools. Available terminal resolved-model and
serving-provider strings are bounded and retained privately on the model step;
absence remains unknown, not independently verified execution provenance.

Completed text pairs are replayed. Turns with real tool calls also retain their
original request, canonical provider continuation, steering, and honest function
results after interruption. Unfinished prose is excluded. No provider-side stored
response IDs, automatic mutation replay, or automatic retries are used. Across
provider or model changes, only ordinary assistant text and reconstructed function
requests/results are replayed: private reasoning, output item IDs and original
provider call IDs are omitted. Function request/result pairs use recorded local
execution IDs for this translation. Compatible same-provider/model continuation
retains its original opaque items only while its generation-time catalog canonical
identity still matches, even if the response reported the unversioned requested ID.
Missing historical canonical ownership cannot establish compatibility with an
available canonical identity. A canonical change between body preparation and HTTP
admission rejects the request before sending; it does not transmit the old private
context or automatically retry. Safe snapshots expose message/run provider IDs,
not private continuation.

SSE parsing handles fragmented UTF-8, CRLF, multiline data, and keepalive comments.
Content types are compared case-insensitively. If the header is missing, empty, or
unusable, the body must still decode as Responses SSE and reach a terminal event;
headerless JSON-like, HTML/XML-like, empty, and non-SSE bodies fail with sanitized
diagnostics. Explicit non-SSE media types remain rejected. No raw headers or
response bodies are included in these diagnostics.
Text/refusal deltas and bounded tool summaries are presented. A missing terminal response or
provider error fails the reply while preserving partial text. Refusal and
incomplete responses remain distinct terminal statuses. Raw provider error bodies,
credential tokens, and reasoning do not cross ACP.

Provider input/output token reports are saved independently of continuation text
and survive resume. Each new assistant reply records a bounded generation ledger:
an unknown entry is checkpointed before inference, then filled only with reported
counts. Session totals include reported superseded/cancelled generations and tool
continuations, even when their content is excluded from future context. Missing
reports remain unknown or a lower bound; steering alone no longer implies a gap.
A pending generation reservation is removed only when the adapter positively
reports that no inference request was attempted. This preserves previous reported
usage and latest context; attempted failures and crash uncertainty stay unknown.
Legacy replies without a ledger use retained per-step reports conservatively.
Safe cumulative and latest-request context counters cross ACP after checkpointing.
Optional model context-window metadata is retained in conversation and preference
snapshots; it is not a ChatGPT account quota.

Settings are embedded, with environment-only overrides:

- `REPLIES__OPENAI__BASE_URL`: `https://api.openai.com/v1`; HTTPS is required except
  loopback HTTP fixtures.
- `REPLIES__OPENAI__TIMEOUT_SECONDS`: 600; must be nonzero. Applies to inference,
  not to abandoning a credential-refresh commit.
- `REPLIES__OPENROUTER__BASE_URL`: `https://openrouter.ai/api/v1`; its origin must
  match the configured/stored OpenRouter issuer before credentials are used.
- `REPLIES__OPENROUTER__TIMEOUT_SECONDS`: 600; nonzero, cancellable inference.
- `APP_DATA_PATH`: `~/.harness`, the only storage-root setting.

Credential resolution and on-demand refresh are described in
[provider connections](provider-connections.md). A cancelled refresh wait stops
promptly; once a rotating refresh exchange begins, cancellation waits for its
verified replacement to be saved, then skips inference. There is no refresh daemon.

## Persistence and crash recovery

Each conversation lives in `<app_data_path>/conversations/<uuid>.json`, with a
separate `<uuid>.lock`. Only canonical lowercase UUIDs are accepted as identifiers;
callers cannot supply paths. Port-owned JSON records stay in infrastructure, domain types have
no Serde dependency, and unknown versions/corrupt records are rejected unchanged.
Creation owns encode-only records, reading owns decode-only records, and updating
owns both directions; no serialized representation is shared between adapters.
Version three adds an optional Griller identity, linked runs and an interruption
gate. Version four adds immutable instruction snapshots, a selected reference and
each run's actual snapshot reference; absent historical instructions remain
unknown. Version five adds required per-message/run provider provenance. Version six
adds exact approved assignment/CoS metadata without dropping version-five fields. Optional
step-level resolved identity/serving provider, generation-time catalog canonical
ownership and catalog canonical/worker capabilities are retained in independently owned records. Versions one through
five remain readable without rewriting; their single stored provider is the only
justified legacy message/run provider when those fields are absent. Mutation
writes version six. See [role guidance](role-guidance.md).
Each run records its own stable identity, agent identity,
model/effort snapshot and lifecycle status. Reading/updating validates associations
and chronological reply/status consistency, including complete run coverage for
Griller messages. An associated Griller requires an absolute stored workspace;
missing/null workspaces are corrupt records, not legacy workspace binding.
Versions one and two retain legacy usage semantics;
invalid counts, bounds and step/report mismatches are rejected. Legacy messages
are not retroactively assigned runs, creators or approvals.

Directories/files are owner-only (0700/0600). Existing unsafe paths and symlink
records are rejected. Secure storage currently requires Unix. Writes use a private
same-directory temporary file, flush/sync, atomic rename, and directory sync.
Tokio handles filesystem I/O; blocking temporary creation/locking/sync use
`spawn_blocking`. Bounded record decoding, nested continuation parsing and validation,
record conversion, cloning for creation, and bounded JSON encoding also run on
blocking workers. Each adapter keeps its own record directions. Credential,
conversation, catalog-cache, and artifact writes share the private replacement helper.
Revision checks reject stale updates before encoding and writing the replacement. Errors at/after the atomic commit point can
mean the replacement already exists; reload rather than blindly resending.

No fsync occurs per token. During a stream the disk record contains the saved user
message and in-progress assistant; partial text remains in memory until a tool or
steering checkpoint, completion, cancellation, or failure. If the process crashes, load acquires the
conversation lock and durably marks an in-progress reply interrupted. Its pending
generation report remains unknown. It does not pretend the reply completed,
regenerate it, or recover uncommitted streamed text. Domain behavior owns pure
reply/tool transitions and uncertainty classification; application keeps explicit
checkpoint-before-effect and result-before-continuation ordering.

A message/reply is limited to 1 MiB, an SSE event to 4 MiB, and a conversation file
to 16 MiB. JSON encoding stops at the file bound rather than building an oversized
buffer first. Before saving requested/running calls, metadata capacity is reserved
for every outstanding result's maximum encoded preview/artifact pair, terminal
status, elapsed counter, truncation flag, and finishing message/revision/timestamp
metadata. Existing arguments and history are already included in the encoded size.
A checkpoint without sufficient capacity fails before the tool starts; a completed
result can consume its reservation without increasing the 16 MiB file bound.
There is no automatic compaction or deletion. These files contain
sensitive plaintext history; protect/back up the app data directory accordingly.

## Catalog caches

Model/provider catalogs retain their TTL memory cache and remote-first disk
fallback. The model adapter independently combines the configured pilot catalog
(`MODELS__URL`) with OpenRouter's public catalog
(`MODELS__OPENROUTER_URL`, default `https://openrouter.ai/api/v1/models`).
OpenRouter entries cannot be supplied by the pilot catalog. Its raw independently
owned response is cached at `openrouter-models.json`. Either source can remain
usable if the other fails; entries from the unavailable source are unavailable
unless its bounded fallback succeeds. Catalog presence/parameter support never
proves account access, inference quality or actual execution. Remote bodies, fallback files, and encoded cache files each have a
16 MiB limit, checked before parsing or growing the encoded buffer past that
limit. Both advertised HTTP lengths and streamed chunks are bounded. Parsing,
cache encoding, memory-cache cloning, and domain conversion run on blocking
workers. Each catalog adapter owns its response/cache representation.

Cache writes are optional, private, and atomic; a failed cache write does not
invalidate a successful remote read. Invalid, unsafe, or oversized fallback files
are rejected. User-facing failures use fixed catalog context labels, and optional
cache-write warnings contain no raw errors, URLs, paths, or response content.

## Explicit resume

```sh
./bin/harness --resume <conversation-id>
```

The frontend parses exactly one ID, and Rust loads it through standard ACP
`session/load`. Bad/missing arguments, absent files, unsupported versions, and
corrupt records are visible startup errors, never a silent fresh fallback. No
argument opens this repository's saved Griller if present; it never picks an
arbitrary legacy conversation.

The `--resume` startup option loads history, provider-connection reference, model,
and effort without sending a prompt; it is not execution authorization. Credentials and catalog availability are not needed for reading.
Sending validates the selected catalog model and resolves the original connection;
if unavailable, choose a model or reauthorize the same account. The transcript
omits identity/provenance headings; PgUp/PgDn still browse saved/current text.
Conversation IDs are available from their filenames.

## Steering

Enter during generation sends a UUID-tagged instruction to Rust. Acknowledgement
follows persistence; duplicate IDs are idempotent. Inference may restart within
the same turn, while a running tool finishes before remaining batch calls are
recorded Skipped. There is no frontend cancel/finalize/resend loop.

Explicit Stop cancels without a follow-up. Escape only closes/back-navigates. Partial prose remains saved but is not
replayed as completed provider output. Actual tool calls/results survive
interruption; recovery marks unfinished mutations Unknown and unfinished reads
Interrupted. Unknown actions are never automatically replayed. Unsubmitted drafts
survive completion.

## Repository Griller and interrupted runs

A canonical Git worktree root identifies a repository context (a non-Git directory
uses its canonical path). Subdirectories resolve to that root. Git output loses
only its single newline terminator; legitimate trailing pathname whitespace is
preserved, never trimmed into a sibling repository. The repository
conversation key is a namespace UUID derived from that path; its actual Griller
agent ID is separately generated and persisted. A repository move is a new context;
linked Git worktrees are separate contexts. Opening an existing Griller retains its
saved connection/model/workspace; no history scan, role inference from prose, or
legacy adoption occurs. An explicit `--resume` continues an old pilot conversation
without inventing an agent association.

The Griller uses the stock provider factory, ordinary tools, and Harness-owned
versioned role/phase instructions, explicit repository guidance and durable snapshots
through the [role-guidance resolver](role-guidance.md). It refines ideas and asks for human input; this slice
can publish reviewable proposals with normal repository-supported tools and capture
them through `publish_work`. A separate trusted exact-version approval operation
creates CoS; Griller prose cannot grant implementation/merge authority. Native
children, review, integration and merge remain unavailable. See
[approved work](approved-work.md). Each submitted answer starts a later grilling run on the same agent;
steering stays within the active run. A successful response ends the run for human
input, not as factory completion. Model steps and generations remain the existing
durable message-level records.

A recovered in-progress reply/run becomes Interrupted and requires explicit
Resume. Read/load/start never dispatch inference or tools. An ordinary prompt
cannot bypass the gate. `/resume` uses the public Resume operation, preserving the
same run/reply and saved model while continuing only from recorded provider items
and honest results. It keeps generation/execution budgets across recovery and
never re-executes pending/interrupted calls.

Unknown tool outcomes block direct continuation. `/resume inspect` explicitly
acknowledges the interruption **without inference**, allowing a new human
inspection instruction/run; it does not mark an effect reconciled or successful.
The Unknown result remains in history. Inspect external facts before deciding
whether to retry a mutation. This is a narrow leaf recovery path, not automatic
reconciliation or an exactly-once remote-effect guarantee.

Agent/run metadata commits in the same bounded atomic conversation replacement
as its messages and tool checkpoints. The existing per-conversation process lock
also serializes repository creation and execution; concurrent owners are rejected.
There is no child execution engine or subtree expansion in this slice. Approved
handoff records a separate CoS and its creator, not execution ownership. Both
agents are leaves; Stop affects only the selected active execution.

## Fresh-session model preference

The last selected/used model and effort are stored as a versioned snapshot in
`app_data_path/preferences/model-selection.json`, using a private directory,
0600 file, atomic replacement, and bounded reads. Domain selection validation is
shared with conversation creation/sends. Application read/update use cases use
closure-backed ports; infrastructure owns serialization and filesystem I/O.

When there is no saved repository Griller, startup reads this snapshot without
loading history or creating a conversation,
checking credentials, fetching catalogs, or starting inference. Picker commits
validate against the catalog and save the selection. Sending also remembers the
validated model/effort actually used, including in resumed conversations. The
frontend serializes preference writes and flushes them before prompts and shutdown.
Across processes, the last committed complete snapshot wins.

A missing preference simply leaves the model unselected. Unsupported, corrupt,
oversized, or unsafe records produce a nonfatal startup warning; reads never
silently overwrite them. Explicitly choosing a valid model can replace a corrupt
regular file. Sending still validates current catalog availability; restoring a
snapshot does not assert the model remains available. Explicit resume uses its own
selection and does not change the fresh-session default merely by loading.

## Validation

Temporary external probes exercise the actual Rust/official SDK/Ink paths:
durable completion/cancellation/failure/refusal/incomplete statuses, fragmented
Unicode SSE, serialized rotating refresh across simultaneous conversations,
cancellation during refresh, prompt overlap, restart/resume without credentials,
crash interruption, eligible history, strict identifiers, private files,
corrupt-file preservation, startup arguments, transcript paging, and 50×18 resize.
No permanent tests or test-only dependencies were introduced.

Real-account browser login and live subscription inference remain manual
verification steps.

Primary provider documentation:

- https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference
- https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions#refreshing-tokens
- https://developers.openai.com/siwc/token-sharing-open-source/token-reference
