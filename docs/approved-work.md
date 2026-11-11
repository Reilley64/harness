# Approved work and separate Chief of Staff

## Implemented slice

A repository Griller can publish a reviewable proposal with normal shared tools,
then capture its exact source document through `publish_work`. A trusted local
human can approve that version and deliver a durable handoff to a separate Chief
of Staff (CoS). CoS uses the same real conversation, model-step and tool loop as
Griller, with its own identity, instructions, messages, runs and accounting.

This module owns approved handoff, not managed implementation. CoS can prepare
and inspect a [native Decider selection](native-decider.md); separate
[managed Research dispatch](managed-dispatch.md) consumes settled selections.
Managed workspaces, Implementer execution, review, integration and target merge
remain unavailable. Unknown tool names produce ordinary Invalid results; unknown
ACP methods are unavailable. No synthetic execution, readiness or spec completion
is produced. `complete` is always false here; a later completion
owner must establish the exact human-authorized verified target merge.

Tools are OS-user trusted, not a sandbox or an effect classifier. Approval does
not introduce a per-role permissions matrix, authorize machine-wide/external
setup, grant live/paid access, or make arbitrary Bash commands safe.

## Publication and source identity

Follow the repository's actual tracker/git-host and draft conventions. Griller
can use existing `read`, `write`, `edit` and `bash` to prepare, publish and read back
artifacts through a configured CLI. This slice adds no tracker-specific adapter,
MCP client, automatic installation, provisioning or credentials. If the documented
path is unavailable, report it and ask for help. A local draft/snapshot is not
remote tracker interoperability or an exclusive tracker claim. Provider OAuth
credentials are never supplied for tracker access.

After the phase's human confirmations, write a UTF-8 proposal JSON document:

```json
{
  "root": "<exact published spec URI or honest local artifact path>",
  "source_revision": "<source revision/version observed at publication>",
  "spec": "<reviewable spec retaining repository sections/conventions>",
  "scope": "<precise implementation scope>",
  "criteria": "<precise acceptance criteria>",
  "dependencies": "<source-linked ticket graph and native dependency proposal>",
  "wiki": "<existing shared wiki index pointer, or null>",
  "context": null,
  "artifacts": ["<durable source/artifact pointer>"]
}
```

`wiki` and `context` are pointers, not fetched semantic memory. Null explicitly
means unavailable/unknown. Reuse existing wiki guidance; this slice neither runs
Setup nor requires a fabricated Setup completion, glossary or ADR gate.

The typed model tool `publish_work` takes `{workId, intent, path, revision?}`.
`workId` and publication `intent` are canonical UUIDs; `path` resolves using the
saved workspace like ordinary file tools. The backend reads the real file,
rejects unknown fields, validates bounds and captures its **exact bytes**, decoded
fields and SHA-256 version. It never treats a model's digest or summary as the
source of truth. Source root/revision remain assertions about the named artifact;
remote read-back must be established using the repository's actual tools.

Proposal capture accepts regular files only. Unix opens are nonblocking before
checking the opened descriptor, so a writerless FIFO or replacement with one
cannot hold Stop hostage. Cancellation follows the file read and is checked
again before admitting a durable publication. Cancellation after an admitted
atomic write is not a promise to undo that write.

First publication omits `revision`. Amendments use the current durable revision
and a new publication intent. Same intent/exact document reuses the publication;
changed payload under that intent conflicts. Work identity, creator, originating
Griller conversation, workspace, original publications and approvals are immutable.
A work identity cannot be repurposed to another source root. Distinct work intents
use distinct explicit work IDs rather than a fuzzy duplicate classifier.

New document versions invalidate affected old authority, conservatively for this
whole work snapshot. No model classifier decides whether a scope change is safe.
Unrelated work IDs remain unaffected. Mutable tracker text, formatting, readiness,
comments or prose cannot silently replace the backend snapshot. Editing an
external artifact alone does not broaden approval: only the exact captured version
is authorized. Publish changes explicitly before requesting new managed work.

## Trusted approval and handoff

Only the trusted local application/ACP approval operation records authority.
Ordinary chat saying “approved”, tracker readiness, model tool calls, handoffs and
summaries cannot call that operation through the model tool set. The local ACP
client and OS user are trusted; Bash is not isolated from that OS user's files.

Inspect `/work <work-id>` in the native launcher. The scrollable inspector shows
work/revision, exact source document including root/scope/criteria/dependencies,
SHA-256 version, retained Griller model/effort, prior approvals and unavailable
capabilities. Press `a`, then explicitly `y` to approve that precise version.
Escape or `n` declines/dismisses; navigation alone never authorizes anything.

The backend checks the exact work version/revision and the originating Griller's
idle conversation revision, role, provider/model and effort. Stale changes or an
interruption reject rather than approving a different target. Current model,
allowlist, compatibility and that provider's own credentials are checked without
inference. No router, default model, lowered effort or credential substitution is
used. A previously recorded exact intent can be retried with its original facts;
a different payload conflicts even if it would otherwise be eligible.

Approval also binds an optional initial transient-retry allowance, 0–64, to the
same exact intent/version. Omitted policy retains default two, including old
approvals; explicit zero and explicit two remain distinguishable from omission.
New CoS and authorized Research retry episodes use that recorded value, without
resetting any existing episode. This is not a whole-work time/spend cap, quota
permission, native-Decider retry, or substitute for later human extension. The
[frontend](frontend/configuration.md#approved-work-and-cos) owns selection and
confirmation controls; [ACP](acp/conversations.md#approved-work-extension) owns
the optional typed request field.

Approval and the assignment-intent claim are committed before CoS creation.
Each explicit assignment intent has a deterministic UUID, separate from its work
identity; same key/payload reuses that identity across processes/restart. A new
intent means a deliberately distinct assignment, not an inferred retry. The native
inspector reuses an existing matching intent by default, including after restart;
it does not manufacture another CoS for repeated clicks. The public query retains
all original approval facts, intent identities and delivered/pending status.

CoS creation is an atomic create-or-read under its own conversation lease. It
records sender, recipient, work/assignment/version, exact approved proposal,
existing wiki/context/artifact pointers and approval time before any continuation.
Its role is Chief of Staff, not a renamed Griller. Griller's transcript and private
provider continuation are never copied. No run or generation is fabricated at
creation: the delivered handoff is available immediately and a human conversation
starts its first actual run through the stock coordinator.

The versioned resolver loads `role/chief-of-staff`, `procedure/coordination` and
their real dependency closure. Captured instructions and provenance use the
existing conversation snapshot machinery. Default unambiguous instructions can be
selected at creation. Explicit local/repository overrides remain an unselected
candidate until the human inspects `/guidance` and explicitly adopts them; sends
block before inference until then. Missing dependencies block handoff visibly,
retaining the already admitted approval/intent for an explicit retry after repair.

The public durable-work read and conversation load paths inspect without inference.
Use `o` after delivery to open CoS, `/open <conversation-id>` to open its existing
conversation, `F3` or `/griller` to return to the repository Griller, and
`./bin/harness --resume <conversation-id>` after restart. Native title and map
metadata identify the selected actual role and recorded creator. Creator identity
is not execution ownership or blanket Stop propagation. The handoff module does
not own child execution; [managed dispatch](managed-dispatch.md) documents the
available controls and incomplete subtree/restart integration. Growing multi-agent maps and
background conversation switching are not implemented by this handoff slice.
Drafts and transcript viewport anchors are retained independently by conversation
within the native UI session, including visits to an empty CoS. Plain empty-prompt
Left supersedes pending conversation navigation just as F2/Escape do.

## ACP/application paths

Initialization advertises `harness/approvedWork: true`.

- `_harness/work/read`: `{id}` → `{work, conversationId: null}`. Work includes
  immutable publication documents/versions, creator/session/workspace, current
  revision, approval facts, deterministic CoS IDs, delivered/current flags and
  unavailable capabilities. Missing work is null; corrupt work is rejected.
- `_harness/work/approve`: `{id, revision, version, intent, sessionRevision,
  providerId, modelId, effort}` → `{work, conversationId}`. This is trusted human
  input, not an advertised model tool. Retry must retain the original revision,
  version, intent and pair. No inference is performed by this request.
- Standard `session/load/prompt/cancel` and explicit Resume/acknowledge continue
  to serve both roles. Safe metadata adds `agent.role: chief_of_staff`, recorded
  `creatorId` and `assignment` source document/work/version/intent/approval time;
  actual run task is `coordination`. Private continuation remains private.

Application owns `create_work`, `read_work` and `update_work` orchestration, each
with private dependencies/error/output children. Closure-backed ports are
`ReadWorkProposal`, `ReadWork`, `UpdateWork` and `CreateCoordinator`, each with its
matching infrastructure adapter and independently owned wire/storage records.
Domain owns proposal bounds, intent/payload conflict, amendment and authorization
rules without Serde, including exact work/creator/coordinator/proposal/approval-time
assignment matching. Application and HTTP admission each independently read the
composed work port before applying that same domain predicate. Reads/changes are bounded, storage parsing/encoding/locking
runs on blocking workers, filesystem I/O uses Tokio, and no work-storage lock or
repository-wide scheduler lock is held across inference.

## Durability, admission and recovery

Records live under `APP_DATA_PATH/work/<UUID>.json` with owner-only bounded atomic
replacement and a separate no-follow process lock. Work files are format 1, at
most 4 MiB, with at most 64 publication and 64 approval entries. Proposal documents
are at most 256 KiB; pointers are at most 4096 bytes with at most 32 artifacts.
Every record read checks exact document/decoded-field/hash equivalence, IDs,
versions, original authority and assignment references. CAS rejects a stale write;
updates cannot remove/change immutable original facts or regress delivery status.
The optional retry-policy field is range-checked independently by each work codec;
absence remains absent on writes, and old approvals are not rewritten as explicit
human choices. Existing serialized fields and proposal document fingerprints keep
their order and meaning.
There is no automatic deletion or compaction.

Conversation schema 6 adds optional approved assignment/CoS metadata while
retaining all v5 provider, canonical-ownership, usage, instruction and continuation
fields. Versions 1–5 remain readable without rewriting; mutations write 6. Legacy
records are never assigned a CoS, approval or fabricated instruction history.
Independent create/read/update port-owned records preserve their directions.

Before sends/Resume, each generation, and each next tool effect, CoS's exact
handoff must still match backend authority. Both worker providers also recheck it
at their shared HTTP admission boundary after credential resolution. A concurrent
change after admission cannot recall an already-started request/effect, but its
usage/results are preserved and subsequent work is blocked. Current model policy
and capability admission are not widened by work approval. Tool execution borrows
only a turn's immutable sender/role/assignment facts plus its saved workspace,
not a full conversation/history/continuation/instruction clone before each tool.
The bounded assignment facts are captured once on entering the turn loop; each tool still reads
current durable authority independently, and steering remains checkpointed while
that operation is pending.

Cancelled UI reads are aborted and late results ignored. The conversation hook
owns the whole Griller lookup, conversation load and model restoration operation;
newer navigation, sends, transport replacement and unmount invalidate it before a
superseded lookup can initiate a load. Interruption acknowledgements update only
their still-selected target at a non-older revision under the same navigation
ownership. A late acknowledgement cannot replace newer history, another actor's
selection or its model; backend acknowledgement is not rolled back by ignoring it. An admitted durable
approval/handoff may finish after dismissal/disconnection; it cannot overwrite a
newer target/view. Query before retrying, keeping the original intent. A pending
claim is not an active run, a completed handoff is not inference, and restart never
automatically replays pending work. Explicit retry reconciles the deterministic
CoS create/read and delivery marker; unknown effects remain unknown.

Actual generation/tool crash recovery, Stop and explicit Resume retain the
existing checkpoint-before-effect, result-before-continuation and provisional
usage rules. A killed request stays unknown unless there is positive no-attempt
proof. Unknown tool effects block direct Resume; inspection acknowledgement neither
replays the mutation nor marks it successful. No inference is launched to invent a
handoff after Stop/crash. Normal waiting is allowed, without polling generations.

## Verification limits

Temporary public application/ACP and stock-launcher PTY probes retain exact inputs,
source documents, effect/read-back oracles, candidate identities, raw requests,
native bytes and failures outside the checkout. Scripted provider/OAuth and tracker
CLI/HTTP boundaries are explicitly substitutions, not Plane/MCP interoperability,
account entitlement, billing/quality or live acceptance. Native byte observations
are not human terminal-emulator visual signoff. The inherited Ink approximately
20 ms incomplete-prefix boundary remains disclosed and unwaived.
