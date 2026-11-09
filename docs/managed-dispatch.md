# Managed Research dispatch

## Runtime boundary

Research executes through the ordinary Rust application coordinator, provider
factory and checkpointed tool loop. A dispatch acknowledgement identifies the
accepted child, assignment and execution; it does not wait for the child's report.
Independent children use independent conversation execution guards. They do not
use T3 agents, a simulated event producer or a fabricated workspace.

Griller may delegate scoped Research with exact-parent model/thinking inheritance.
An approved Chief of Staff uses its prepared, settled native selection or recorded
exact-parent fallback. The native Decider remains a separate typed operation.
Implementer execution is unavailable until managed assignment workspaces exist;
review, integration, target merge and the growing native map remain separate work.
Tools remain trusted OS-user operations, not a sandbox or role permission matrix.

[ACP child coordination](acp/children.md) owns wire shapes. [Frontend guidance](frontend.md)
owns native interaction; the model-facing controls use ordinary tool panels.
The stock inspector opens existing Research history read-only and exposes fixed
joins, backgrounding, source reports, scoped Stop and pending guidance adoption.
This is not the later growing-map or general execution-subtree integration.

## Admission and immutable assignment facts

`dispatch_research` takes the same bounded brief as `prepare_assignment`, plus
`routed`. It accepts Research only. The actual parent must have a current active
run, saved workspace and permitted role. CoS work must still match its exact
approved handoff. A routed brief must match the settled native intent; an
unrouted Griller captures its actual provider connection, model and thinking.
Current permission, positive Chat compatibility, context, connection and canonical
identity are checked before execution. Every later model admission reloads the
current catalog and rechecks its context capacity against the immutable brief; an
insufficient refresh blocks without replacing the pair or replaying prior effects.
There is no worker fallback after dispatch.

The durable claim retains parent/run/revision, work/version/approval references,
chosen pair, exact brief and Research instruction snapshot. Routed Research also
requires the complete approved-work identity, intent, version and approval time at
every admission boundary; omission or nulling is incomplete authority, never
unrestricted inheritance. Unrouted Griller inheritance carries no work tuple.
Mutable picker preferences do not select a later worker for an accepted intent. The same globally claimed
intent with the same typed payload returns the accepted handle without starting
another future; conflicting payload reuse, including another parent, fails. Intent UUIDs are explicit, not
fuzzy semantic deduplication. The intent UUID is the global execution-claim key.

Default `role/research` and `procedure/research` use the existing versioned,
licensed bundle and dependency resolver. No personal/global skills are read.
Required missing guidance blocks. Explicit local/repository overrides require
inspection and adoption for the exact pending intent, parent conversation revision
and available snapshot hash. Adoption captures parent-agent and approved-work
identity/version in an immutable closure; a changed parent, work version, available
hash or already-accepted intent rejects the action. The claim and adoption share
intent locking. Later override changes affect neither an accepted child nor its
parent. Removing an override is not adoption; active snapshots never change
automatically.

## Claims, joins and receipts

`APP_DATA_PATH/coordination/<parent conversation UUID>.json` is an independent
format-two record containing the distinct durable agent and conversation IDs,
bounded at 4 MiB, with at most 256 retained children and 256 captured joins. These
are storage bounds, not concurrency slots; exceeding required capacity blocks
rather than dropping authority. The matching read/update adapters own independent
records. Current-format authority and Stop facts are required, including immutable
claim Stop fields; omission is rejected, never defaulted into permission.
Private no-follow process locks cover only short read/transition/replace
operations. Bounded conversion/validation/encoding runs on blocking workers; no
model or tool future runs under a storage lock.

An exact immutable claim at `coordination/intents/<intent UUID>.json` persists
before parent admission and before a real child future starts. Per-intent locks
serialize the same key, not unrelated assignments. A claim left without its
parent admission becomes an inspectable Unknown receipt on exact-key recovery;
it is never treated as proof of unsent work or silently replayed. A bounded
per-parent reference index at `coordination/parents/<parent>/<intent>.json`
permits claim discovery without scanning an unbounded global pool. An origin
marker detects loss of initialized coordination. Reconstructed orphan facts have
`authorityComplete: false`: original join/admission facts are not invented and no
fresh prompt can permissively reopen that incomplete authority.

Claims persist before a real child future starts. Each child has a stable
controlling execution identity. Parent coordination has its own controller/run
association and records Waiting before suspension and Running before generation. Transitions validate ownership and advance the durable revision.
Same-key receipt retries are deduplicated; changed outcomes conflict. Distinct
child claims and receipts apply to fresh locked state rather than overwriting a
stale whole-record snapshot. There is no repository-wide scheduler or fleet cap.

`join_children` captures an explicit fixed execution set and `any` or `all`.
Later children do not enlarge it. Failure, Stop, Unknown or input returns partial
state for reconciliation, not success or automatic sibling cancellation.
`background_children` releases the named wait without replacing or restarting its
children. Tool acknowledgement is checkpointed before the runtime suspends the
same parent run. No long provider tool-result wait or inference polling implements
the join. Suspended time is excluded from the parent's active-turn deadline;
individual model/tool limits and the normal generation budget remain in force.

Research execution cancellation is independent of a completed parent prompt's
transport lifetime. A parent NeedsHuman boundary does not cancel unaffected
children. Live workers observe durable parent gates and per-execution Stop fences without
provider polling. Stop fences are monotonic across later parent prompts; a new
parent run cannot revive an old worker. Already-known completed reports and
uncertain effects remain distinct. The backend owns worker futures and independent
tokens. Native exit waits for pending child controls, requests durable Stop, cancels
and drains owned futures before closing stdio; backend shutdown also drains. A
runtime-close race after fresh claim admission retains a no-start Stopped receipt.
This does not imply control of another process's unrelated owned workers.

A child outcome is written independently while the parent is busy. Receipt,
delivery to a parent run and required follow-up completion are separate facts.
`complete_children_followup` explicitly identifies delivered receipt IDs after
handling their report/effects/uncertainty. A terminal parent response cannot mark
that disposition complete; an unfinished disposition remains gated for human
continuation rather than generating model polling.
The active parent consumes eligible outcomes at safe model boundaries, preserving
provider request/result pairing. A completed parent model response can be shown
while owned children remain outstanding; it is not settlement. The owning run
waits without inference and resumes for eligible results or accepted steering.
Closed coordination gates retain outcomes without permitting a new model call.

## Current work and retrieval

Before an eligible parent generation, the application reconstructs a current-work
view from durable child/assignment/execution identities, status/gates, fixed joins,
receipt/source references and required follow-up. A child's durable Stop fence is
explicit even while its status remains Running and its terminal receipt is pending.
It contains no child transcript
copy or child-authored authority. The view is bounded at 64 KiB; missing or
oversized required facts block inference actionably instead of being truncated.
It is request context, not an unlocked conversation-file control channel, durable
summary replacement, semantic-memory service or compaction dependency.

`read_children` and `read_child_result` require no inference. Results retain a
bounded 8 KiB summary and source conversation/run/reply references, with at most
32 artifact references of 4096 bytes. Larger report text stays in its source
conversation. Child text is explicitly untrusted data: it cannot approve work,
settle the parent, change captured joins, rekey intent, replay effects or authorize
merge. Result receipt alone never establishes those authorities. An ordinary
`read_children` tool must not report an empty child set when required projection
facts are missing or oversized; bounded reports remain independently inspectable.

Worker diagnostic chains are retained privately before a receipt at
`coordination/diagnostics/<execution UUID>.json` (format one, at most 4 MiB and
256 causes per chain). Public artifacts expose an opaque diagnostic source URI,
not its contents, credentials or provider continuation. Persistence failures keep
the original errors in the owned runtime failure instead of dropping them. A fatal
owned worker fences its own execution Unknown without inventing a receipt; the
parent does not wait indefinitely on a falsely live Held/Running claim. Private
snapshot reads reopen an inode atomically replaced before metadata validation,
at most eight times; mode, owner, no-follow, hardlink and size checks remain intact.

## Compatibility and recovery limits

Research conversations use schema seven for explicit parent/assignment/execution
provenance. Existing Setup/Griller/CoS mutations continue to write schema six;
versions one through six, private continuation, canonical ownership, usage,
instruction snapshots and exact approved-work authority remain readable.
Older executables reject schema-seven Research records rather than inventing a
legacy association or destructively downmigrating them. Coordination format two
is additive; no existing provider, preference, decision or approved-work records
are rewritten by its creation. The native decision record remains schema one. Earlier unaccepted development
coordination/claim format-one records are rejected unchanged rather than inferring
missing durable agent-versus-conversation identity or downmigrating them.

A held or running claim is not proof that a request was unsent. PID death or an
expired heartbeat cannot authorize replay. Reopening does not launch an accepted
child again. Normal leaf recovery retains uncertain effects and requires explicit
Resume. Recovery of a controlling parent gates continuation and durably fences
unfinished Research as Unknown without inventing a terminal report for a dead
worker. A late actual worker receipt can still attach its known source facts.
Known receipts, delivery and incomplete follow-up survive separately. Only
explicit same-run Resume admits a new parent controller; acknowledging uncertain
work instead fences the old scope and enables a later human instruction, not an
old effect replay. A missing report remains unresolved, not settled.

Exact-key orphan inspection retains Unknown facts without rechecking a new worker
selection or launching a future. Active parent wait loops observe cross-process
receipts and wake at safe boundaries, not through inference polling. There is no
always-on scheduler or execution while closed. Full subtree/later-run/input and
approval integration, complete idle-process arbitration and the composed native
map remain HARNESS-30/31/42 work, not claims implied by these available controls.

Controlled development probes exercise real Rust workers and tool writes across
competing backend processes, fixed joins/backgrounding, parent input boundaries,
exact-key orphan inspection and separate receipt/delivery/follow-up. Only loopback
catalog/provider responses and synthetic credentials are substituted. These are
not final qualification, full restart/wake arbitration, live quality, paid access
or whole-MVP acceptance. Additional development probes traverse public application
bundles, routed/fallback CoS, busy model/tool receipt boundaries, actual SDK/Ink
launcher paths, real process kills before continuation and between delivery and
follow-up, private-record races and available admission gates. Each substitution
and inherited/native limitation stays explicit; ACP alone does not prove Ink.
