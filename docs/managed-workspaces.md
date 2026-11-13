# Managed implementation workspaces

## Approved dispatch

The existing public `create_dispatch` use case and
`_harness/children/dispatch` accept `role: "implementer"` only for the actual
approved Chief of Staff with `routed: true` and its exact settled assignment
brief. The model tool keeps its compatibility name `dispatch_research` and
advertises both roles. A tracker status, inherited Griller pair, child report or
arbitrary workspace path cannot authorize implementation. Review, integration,
Retro, staged cleanup and target merge remain separate operations.

Implementer `toolContext` is a bounded JSON string, captured by routing and the
immutable execution claim:

```json
{"integration":"refs/heads/spec-integration","setup":["AGENTS.md"],"checks":["documented required check command"]}
```

`integration` must name an existing local branch. The first assignment pins the
spec's repository/integration-branch association; another assignment for that spec
cannot silently replace it. Each assignment captures the branch's actual base
commit when preparing its own workspace. Distinct intent UUIDs produce distinct
assignment, agent, branch and worktree identities even for the same ticket.
Selected model/thinking, approval, instruction snapshot and existing retry policy
remain immutable. Missing guidance or unadopted overrides block, never select a
generic Implementer prompt. `/children guidance <intent>` inspects the phase from
the prepared intent; explicit adoption captures its implementation closure under
the exact parent/work/revision/hash, without changing an accepted intent.

## Native ownership and setup

Records and evidence live outside the removable checkout at
`APP_DATA_PATH/workspaces/<assignment UUID>/workspace.json`. The checkout is its
`checkout` child. Branches are `refs/heads/harness/<work UUID>/<assignment UUID>`;
an immutable `refs/harness/assignments/<assignment UUID>` base marker accompanies
branch creation in one native Git ref transaction. Existing foreign refs are not
adopted. No operation resets or checks out the original repository worktree.

A planned record precedes Git mutation. Separate refs-pending and tree-pending
checkpoints precede their effects; ready/submitted records follow native readback.
An operation lease serializes one assignment's Git/check operation, separately
from short atomic record writes. It is not a repository scheduler lock and does
not serialize unrelated assignments or inference.

`setup` lists required relative workspace references, not commands to provision
services. Each must resolve inside the owned checkout. Agents follow repository
setup through ordinary tools. Missing ignored fixtures block before inference;
Harness does not copy secrets, mutate shared services, invent a conflict detector
or fall back to the original checkout. Worktrees do not isolate processes, ports,
databases, caches or external services. Check-command failures can expose actual
resource conflicts but do not identify every possible conflict. A missing setup
reference produces an explicitly safe required-workspace-setup diagnostic with an
inspection direction; original I/O causes remain private and retained.

## Exact submissions and handoff

The Implementer commits through ordinary tools, then calls
`submit_implementation` with the full native HEAD hash, not `HEAD`, a branch name
or a claimed success object. Harness checks the ownership marker, attached branch,
common repository, clean tracked/untracked state and base ancestry. It checkpoints
check intent, runs the nonempty captured required commands in that workspace,
retains each native status/output, and verifies cleanliness and unchanged HEAD
again before recording success. Failed, cancelled, truncated or missing checks
are not passes. Known failed checks remain eligible for a new submission. A native
Unknown result retains the `checking` phase and its original check evidence,
returns an Unknown submission tool result and stops the turn. It cannot become an
ordinary failure that permits another same-HEAD submission. Inspection and restart
retain that fence; Resume does not replay an uncertain check. In-flight checks
after interruption are likewise uncertain, not replayed.

Submission results persist before tool acknowledgement. Successful submission
ends the run at a typed human-follow-up boundary, without another model call. The
source conversation, exact commit, check record and sender/recipient assignment
identities survive outside the checkout. A handoff is not review acceptance,
tracker closure, integration or merge permission. A model response without a
confirmed current submission cannot become a Completed implementation receipt.
Moved/dirty/foreign workspaces invalidate the current projection rather than
qualifying a different revision with earlier checks. Persisted submission hashes
are validated, and the latest submission must match the recorded native tip and
checking/submitted lifecycle. Earlier historical submissions keep their own hashes
rather than being rebound to the latest tip.

`_harness/workspace/read` takes `{sessionId, executionId}` for the owning CoS and
returns `assignmentId`, `path`, `branch`, `base`, `tip`, nullable `submission`,
`checks` (`command`, native `status`, retained `output`) and an artifact URI. This
uses the public `update_workspace` Inspect action without inference or Git
mutation. Errors preserve the durable record; they are not successful empty
submissions. Explicit tool/check output may contain sensitive data under the
existing trusted-tool contract.

## Rework and recovery

The matching typed answer starts a later run on the same Implementer and
assignment; ordinary prompts cannot bypass input, approval, Stop or restart gates.
After a submitted run, that answer consumes one of the two additional recorded
implementation cycles (three including the initial cycle). The pure domain cycle
transition owns allowance, source-run deduplication and exhaustion; application
orders workspace admission, while the adapter atomically retains that transition
under the assignment operation lease. The source run ID
deduplicates admission across recovery. Routing and allowances never reset by
recreating a workspace. Exhaustion blocks for explicit human extension; an
extension interface and independent review policy are not implemented here.

An absent, properly unregistered workspace can be recreated only with its native
ownership marker and unchanged recorded branch tip. Existing paths, dirty work,
foreign ownership and still-registered missing paths are preserved for inspection.
The implementation path itself performs no staged cleanup.

Inspection of an interrupted Implementer checks its real execution lease before
marking an unowned active claim recoverable. It performs no model call or Git
mutation. Explicit Resume rechecks ordinary ownership/scope/Stop/approval gates
and reconciles native branch/worktree facts. A completed worktree creation can be
recognized after mutation-before-acknowledgement without issuing another add.
Missing, foreign or ambiguous facts block; an absent ref is not permission to
repeat a ref transaction. Unknown tool effects retain the existing reconciliation
gate. This is not exactly-once Git, model or remote execution.

Implementer conversations write schema nine with explicit role, execution
association and the existing schema-eight retry/accounting facts. The historical
`research` execution-association wire field is retained for compatibility, not a
claim that the durable role is Research. Independent read/update/create records
validate the actual role/phase pairing. Older executables reject schema nine;
existing conversations retain their original roles and authority.

## Qualification limits

Temporary external public-use-case probes retain real coordination storage,
Git/worktrees, tool execution, native check outcomes and process interruption.
Approval, catalog, settled native routing and model generation may be explicitly
substituted ordinary ports. These controls do not prove those substituted
adapters, live model quality/entitlement, the stock launcher, composed ACP or
native interaction. Each such boundary requires its own retained evidence; source
wiring and compilation alone are not semantic acceptance. No review, integration,
cleanup, live-resource or whole-MVP acceptance follows from this slice.
