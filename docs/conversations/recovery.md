# Conversation persistence and recovery

## Persistence and crash recovery

Each conversation lives in `<app_data_path>/conversations/<uuid>.json`, with a
separate `<uuid>.lock` for execution ownership and `<uuid>.mutation.lock` for
short storage transactions. Only canonical lowercase UUIDs are accepted as identifiers;
callers cannot supply paths. Port-owned JSON records stay in infrastructure, domain types have
no Serde dependency, and unknown versions/corrupt records are rejected unchanged.
Creation owns encode-only records, reading owns decode-only records, and updating
owns both directions; no serialized representation is shared between adapters.
Version three adds an optional Griller identity, linked runs and an interruption
gate. Version four adds immutable instruction snapshots, a selected reference and
each run's actual snapshot reference; absent historical instructions remain
unknown. Version five adds required per-message/run provider provenance. Version six
adds exact approved assignment/CoS metadata and Setup roles/tool drafts without
dropping version-five fields or inventing historical human consent. Optional
step-level resolved identity/serving provider, generation-time catalog canonical
ownership and catalog canonical/capability snapshots are retained in independently
owned records. The additive `capabilities` array contains `chat` and/or `decisions`;
absent historical arrays remain empty/unknown. Obsolete boolean metadata is ignored,
not converted into permission. This optional metadata preserves version-five
provider provenance, instruction snapshots and version-six assignment authority.
Versions one through five remain readable without rewriting; their single stored
provider is the only
justified legacy message/run provider when those fields are absent. New updates write version eight with explicit provider-retry authority/history and
generation cost/accounting facts; initial empty records may still be six/seven.
Older versions remain readable without inventing facts. See
[provider waits and usage](../provider-waits.md). Research retains its explicit
schema-seven lineage in eight as well.
See [managed Research compatibility](../managed-dispatch.md#compatibility-and-recovery-limits),
[role guidance](../role-guidance.md) and [repository Setup](../repository-setup.md).
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
blocking workers. Model request preparation takes ownership of its conversation
instead of copying full history before the blocking builder. Only model/effort
and assignment identity facts needed for fresh HTTP admission are retained;
necessary metadata copies run on a blocking worker too. Each adapter keeps its
own record directions. Credential,
conversation, catalog-cache, and artifact writes share the private replacement helper.
Every conversation writer takes the separate per-record mutation lock, including
initial creation, controller checkpoints and live retry controls. The revision
check, bounded encoding and replacement are one short critical section;
contention or a stale revision rejects the update for fresh inspection instead
of losing acknowledged facts. Execution ownership remains separate, and no model
or network work runs under the mutation lock. Errors at/after the atomic commit point can
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

## Explicit resume

```sh
./bin/harness --resume <conversation-id>
```

The frontend parses exactly one ID, and Rust loads it through standard ACP
`session/load`. Bad/missing arguments, absent files, unsupported versions, and
corrupt records are visible startup errors, never a silent fresh fallback. No
argument opens the detected repository Setup/Griller if present; it never picks an
arbitrary legacy conversation.

The `--resume` startup option loads history, provider-connection reference, model,
and effort without sending a prompt; it is not execution authorization. Credentials and catalog availability are not needed for reading.
Sending validates the selected catalog model and resolves the original connection;
if unavailable, choose a model or reauthorize the same account. The transcript
omits identity/provenance headings; PgUp/PgDn still browse saved/current text.
Conversation IDs are available from their filenames.

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
through the [role-guidance resolver](../role-guidance.md). It refines ideas and asks for human input; this slice
can publish reviewable proposals with normal repository-supported tools and capture
them through `publish_work`. A separate trusted exact-version approval operation
creates CoS; Griller prose cannot grant implementation/merge authority. [Managed Research](../managed-dispatch.md) owns child execution and its remaining
qualification limits. Implementer execution, review, integration and merge remain
unavailable. See
[approved work](../approved-work.md). Each submitted answer starts a later grilling run on the same agent;
steering stays within the active run. A successful response ends the run for human
input, not as factory completion. Model steps and generations remain the existing
durable message-level records.

A recovered in-progress reply/run becomes Interrupted and requires explicit
Resume. A parent already ended for human input is also gated if its actual owned
work lost its controller. [Idle-owner recovery](../managed-dispatch.md#idle-owner-recovery)
uses per-execution ownership leases, not PID death as proof of unsent work.
Explicit Resume at that ended boundary restores the existing question without
inference; only its matching later answer starts a new run. Read/load/start never dispatch inference or tools. An ordinary prompt
cannot bypass the gate. `/resume` uses the public Resume operation, preserving the
same run/reply and saved model while continuing only from recorded provider items
and honest results. It keeps generation/execution budgets across recovery and
never re-executes pending/interrupted calls. If the saved last checkpoint already
contains a successful typed human-input request, explicit Resume restores that
request without inference instead of generating through it. Only the subsequent
matching typed answer begins a later run; the question is not silently dropped.
A staged answer whose owner still retains the exact original input receipt can
restore that question only under the [unadmitted-answer proof](../managed-dispatch.md#typed-human-input-and-later-runs).
An interrupted restoration preserves its Resume gate; missing generation-accounting
facts or any started work prevent destructive reconciliation.

Unknown tool outcomes block direct continuation. `/resume inspect` explicitly
acknowledges the interruption **without inference**, allowing a new human
inspection instruction/run; it does not mark an effect reconciled or successful.
The Unknown result remains in history. Inspect external facts before deciding
whether to retry a mutation. This is a narrow leaf recovery path, not automatic
reconciliation or an exactly-once remote-effect guarantee.

Agent/run metadata commits in the same bounded atomic conversation replacement
as its messages and tool checkpoints. The existing per-conversation process lock
also serializes repository creation and execution; concurrent owners are rejected.
This leaf recovery path does not own managed child wake/restart. A durable
[ownership-scoped Stop](../managed-dispatch.md#ownership-scoped-stop) cannot be
reopened by same-run Resume; inspect/acknowledge first, preserving old execution
fences before a later human instruction. Approved handoff records a separate CoS and its creator, not execution
ownership. [Managed dispatch](../managed-dispatch.md#compatibility-and-recovery-limits)
owns the current child-execution boundary; typed later-run and child Resume controls are owned by managed dispatch; their
complete composed qualification remains a separate evidence obligation.

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
