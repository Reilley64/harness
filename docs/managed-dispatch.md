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
The current stock UI does not yet provide a child-status/join inspector or active
worker navigation. Those missing controls are not implied by the ACP capability.

## Admission and immutable assignment facts

`dispatch_research` takes the same bounded brief as `prepare_assignment`, plus
`routed`. It accepts Research only. The actual parent must have a current active
run, saved workspace and permitted role. CoS work must still match its exact
approved handoff. A routed brief must match the settled native intent; an
unrouted Griller captures its actual provider connection, model and thinking.
Current permission, positive Chat compatibility, context, connection and canonical
identity are checked before execution. There is no worker fallback after dispatch.

The durable claim retains parent/run/revision, work/version/approval references,
chosen pair, exact brief and Research instruction snapshot. Mutable picker
preferences do not select a later worker for an accepted intent. The same globally claimed
intent with the same typed payload returns the accepted handle without starting
another future; conflicting payload reuse, including another parent, fails. Intent UUIDs are explicit, not
fuzzy semantic deduplication. The intent UUID is the global execution-claim key.

Default `role/research` and `procedure/research` use the existing versioned,
licensed bundle and dependency resolver. No personal/global skills are read.
Required missing guidance blocks. Explicit local/repository overrides currently
block dispatch rather than being silently adopted. Pending-assignment inspection
and explicit adoption remain an implementation gap; removing an override is not
an adoption workflow. Active instruction snapshots never change automatically.

## Claims, joins and receipts

`APP_DATA_PATH/coordination/<parent conversation UUID>.json` is an independent
format-two record containing the distinct durable agent and conversation IDs,
bounded at 4 MiB, with at most 256 retained children and 256 captured joins. These
are storage bounds, not concurrency slots; exceeding required capacity blocks
rather than dropping authority. The matching read/update adapters own independent
records. Private no-follow process locks cover only short read/transition/replace
operations. Bounded conversion/validation/encoding runs on blocking workers; no
model or tool future runs under a storage lock.

An exact immutable claim at `coordination/intents/<intent UUID>.json` persists
before parent admission and before a real child future starts. Per-intent locks
serialize the same key, not unrelated assignments. A claim left without its
parent admission becomes an inspectable Unknown receipt on exact-key recovery;
it is never treated as proof of unsent work or silently replayed.

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
children. Live workers observe the durable parent Stop gate without provider
polling; already-known completed reports and uncertain effects remain distinct.

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
receipt/source references and required follow-up. It contains no child transcript
copy or child-authored authority. The view is bounded at 64 KiB; missing or
oversized required facts block inference actionably instead of being truncated.
It is request context, not an unlocked conversation-file control channel, durable
summary replacement, semantic-memory service or compaction dependency.

`read_children` and `read_child_result` require no inference. Results retain a
bounded 8 KiB summary and source conversation/run/reply references, with at most
32 artifact references of 4096 bytes. Larger report text stays in its source
conversation. Child text is explicitly untrusted data: it cannot approve work,
settle the parent, change captured joins, rekey intent, replay effects or authorize
merge. Result receipt alone never establishes those authorities.

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
Resume. Exact-key orphan inspection retains an Unknown receipt without rechecking a new
worker selection or launching a future. Complete claim discovery/reconciliation,
cross-process wake/restart integration, subtree Stop and later-run/input
integration are not implemented by this candidate;
HARNESS-30 owns their remaining composition. The available parent wait loop is
not evidence that all idle-process/restart wake arbitration has been completed.

Controlled development probes exercise real Rust workers and tool writes across
competing backend processes, fixed joins/backgrounding, parent input boundaries,
exact-key orphan inspection and separate receipt/delivery/follow-up. Only loopback
catalog/provider responses and synthetic credentials are substituted. These are
not final qualification, CoS/native-routing coverage, full restart/wake
arbitration, SDK/Ink launcher evidence, live quality, paid access or whole-MVP
acceptance.
