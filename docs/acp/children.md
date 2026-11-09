# Child coordination over ACP

Initialization advertises `_meta["harness/children"]: true`. These Harness
extensions use stable ACP v1 JSON-RPC, not invented standard ACP methods.
[Managed dispatch](../managed-dispatch.md) owns runtime policy and current limits.
The ordinary `session/prompt` and typed model/tool loop remain the coordinator path.
No provider continuation, provider call IDs, credentials or internal controller
claim token is returned by these methods.

## Requests

Fields are camelCase and unknown fields are rejected.

| Method | Parameters | Effect |
| --- | --- | --- |
| `_harness/children/read` | `{sessionId}` | Read durable parent status without inference. |
| `_harness/children/result` | `{sessionId, executionId}` | Read that parent's bounded source-linked receipt. |
| `_harness/children/dispatch` | `{sessionId, revision, intent, routed, role, task, constraints, toolContext, contextTokens}` | Admit or reuse a Research execution and return before long child completion. |
| `_harness/children/steer` | `{sessionId, executionId, id, text}` | Durably order a targeted human instruction for an existing active owned child. |
| `_harness/children/scope/request` | `{sessionId, executionId, id, text, replaces?, supersedesRun?}` | Persist a targeted scope version and pause its actual ownership subtree pending approval. |
| `_harness/children/scope/approve` | `{sessionId, id, version, approvalId}` | Approve only the latest exact scope version, without satisfying unrelated gates. |
| `_harness/children/join` | `{sessionId, id, mode, executions}` | Capture a fixed `any`/`all` set on the parent's active logical run. |
| `_harness/children/background` | `{sessionId, id}` | Release that captured join without restarting children. |
| `_harness/children/stop` | `{sessionId, executionId}` | Stop this parent's exact execution and its retained owned descendants, not siblings. |
| `_harness/execution/stop` | `{sessionId, executionId?}` | Stop the selected conversation's owned subtree, or its exact selected child subtree. |
| `_harness/children/drain` | `{}` | Stop/cancel/drain this backend's owned workers before shutdown; terminal runtime close, not a fleet Stop. |
| `_harness/children/guidance/adopt` | `{sessionId, revision, intent, snapshot}` | Explicitly capture the inspected pending Research closure. |
| `_harness/children/follow-up` | `{sessionId, outcomes}` | Explicitly record completed required parent follow-up for exact delivered receipt IDs. |

`intent`, `sessionId` and execution/assignment identities are canonical UUIDs.
Dispatch `revision` is the inspected parent conversation revision, not a tracker
revision. `role` must be `research`; Implementer is unavailable. Task is at most
4096 UTF-8 bytes; constraints/tool context are each at most 2048; required context
is 1..1,000,000,000, not a measured token prediction. CoS uses `routed: true` with
its matching settled native intent. Permitted Griller inheritance uses false.
Join membership is nonempty, at most 256 accepted execution IDs and immutable
under the same join ID. Later dispatches cannot enlarge it.

Example read:

```json
{"jsonrpc":"2.0","id":"children-1","method":"_harness/children/read","params":{"sessionId":"<actual parent UUID>"}}
```

The JSON-RPC `id` correlates one request/response. The explicit intent, child,
assignment, execution, parent run and join IDs correlate durable work; a new
JSON-RPC ID does not authorize another execution or a new assignment payload.

## Typed human answer

`_harness/agent/answer` takes `{sessionId, requestRun, text}` and returns the
ordinary streamed prompt response. Inspect/open the existing conversation first;
it must be registered in this connection. The exact requesting run must still
need human input. A fresh run begins on that same durable agent. Parent messages,
targeted updates and child reports cannot substitute for this operation.
Child execution summaries expose `priorReceipts`, including source run/reply and
separate delivery/follow-up fields; full history remains in the source conversation.

## Exact scope approval

Scope request/approval return `{scope, continuationStarted}`.
Scope has `id`, `executionId`, `version`, `text`, nullable `replaces`,
`supersedesRun` and `approvedBy`. Request, version, approval and predecessor
identities are canonical UUIDs. Criteria are nonblank, at most 8192 UTF-8 bytes.
The backend assigns the immutable version and reuses an exact repeated request.
Every later version explicitly replaces the latest request. An obsolete human
question must be superseded by its exact run, not ordinary update prose.

Read exposes the complete retained `scopes` array, including superseded requests.
Approval identifies the exact latest request/version. `continuationStarted`
describes an owned later-run future admitted for an explicitly superseded input;
it is not a settlement, no-effect or completed-inference claim. An already active
controller consumes the approved ordered instruction without a replacement future.
Missing ownership and Stop/Resume gates remain errors, not inferred permission.
[Managed dispatch](../managed-dispatch.md#exact-execution-scope-amendments) owns
runtime semantics and the root-work authority boundary.

## Targeted instruction acknowledgement

`_harness/children/steer` takes the owning parent's session, exact execution and
canonical instruction UUID. Text is 1..16384 UTF-8 bytes and cannot be blank.
It returns `{instruction, effectiveBoundary}` only after durable acceptance.
Instruction has `id`, `executionId`, parent-global `order`, `text`, nullable
`appliedRun` and `appliedReply`. The latter identify a separately checkpointed
application, not a promise that an already admitted effect was undone.
Coordination reads expose the same ordered `instructions` array.

Only the existing immediate-steering policy applies. An active generation is
superseded; an already admitted tool may finish before remaining stale calls
are skipped. Input/approval/restart/Stop require their matching controls;
this endpoint does not enqueue a later run or grant scope authority.

## Responses

Coordination operations return `{coordination, execution, result}`, using null for
fields not supplied by that operation. Execution Stop additionally advertises
`harness/executionStop: true` and returns `{coordination, executions}`: the fenced
scope's coordination and visited execution IDs, not a completed-effects or
settlement assertion. It does not expose controller tokens. Missing required
ownership is an error rather than a successful empty Stop. See
[ownership-scoped Stop](../managed-dispatch.md#ownership-scoped-stop).
Drain returns `{stopped}`; guidance adoption
returns `{intent, snapshot}`, where `snapshot` is the exact inspected available
hash. Missing coordination or receipt is null, not a
fabricated child, successful completion or zero-work assertion. Corrupt storage
fails through the normal safe conversation error boundary (`-32004`); malformed
requests use `-32602`.

`execution` has `intent`, `assignmentId`, `agentId`, `executionId`, `creatorId`,
`parentConversationId`, `parentRunId`, `parentRevision`, nullable `workId`/`workVersion`, `role`, `providerId`,
`modelId`, nullable `effort`, `instructions`, `routed`, `status`, nullable
`outcomeId`, `deliveredTo`, `followUpCompletedBy`, `stopRequested`, `resumeRequired`, and a `source` conversation URI.
Status is `held`, `running`, `completed`, `needs_human`, `failed`, `stopped` or
`unknown`. Creation lineage is separate from actual Stop ownership.

`coordination` has `agentId`, `conversationId`, `revision`, `authorityComplete`,
`gate`, `status`, nullable
`runId`, `settled`,
`children` (execution summaries), and `joins`. `status` distinguishes `running`,
`waiting`, `ready`, `needs_human`, `approval`, `stopped`, `resume_required` and
`unknown`; these are coordination facts, not independent process-liveness proof.
`settled` concerns retained child follow-up, never approved-work acceptance.
Read additionally includes
`contextBlocked` (required facts/diagnostics block the projected context) and
`sessionRevision`, the current atomic conversation snapshot revision needed for
a new dispatch. Reading this snapshot does not recover or steer a live
conversation or start a second controller. Claim discovery may conservatively
retain Unknown orphan facts; this never launches a child. A join has `id`, `runId`,
`mode`, fixed `executions` and `backgrounded`. Gate is `open`, `needs_human`,
`approval`, `stopped`, `resume_required` or `unknown`; these are domain states,
not a claim that every transition is already integrated in this candidate.

`result` has `execution`, nullable `receipt` and an authority warning. Receipt has
`id`, `status`, `conversationId`, nullable `runId`/`replyId`, `untrustedSummary` and
`artifacts`. The summary is bounded at 8192 bytes; source links identify retained
evidence rather than fetching it or granting approval. Receipt, delivery and
follow-up completion are distinct fields, not aliases for settlement. Follow-up
requires its typed control or `complete_children_followup` tool; neither child
prose nor the ending of a parent response writes that disposition.

Safe conversation metadata additionally exposes `agent.role: "research"`, its
actual `creatorId`, and nullable `research: {parentId, parentAgentId, intent, assignmentId,
executionId}`. Research run task is `research`. Existing schema-six Setup/CoS
metadata and private-state exclusion remain intact.

## Admission, cancellation and shutdown

Application use cases own admission, immutable selection, revisioned claims,
result delivery and joins; handlers translate requests and run connection-owned
operations. Reads perform no inference or tool execution. Dispatch checks its
actual parent/run and exact work authority rather than trusting supplied role
text. Scope approval remains the separate trusted approved-work operation.

Cancelling a selected prompt signals that prompt's execution token. Standalone
extension requests may already have admitted a write when a caller stops waiting;
response-wait cancellation is neither rollback nor child Stop. Once an accepted
handle exists, use the scoped Stop control. Join/background await their atomic
storage operation; cancellation is not an inverse mutation. Lost responses require read/reconciliation with the
original keys, not a new intent or blind effect replay.

The ordinary session cancellation/shutdown path still owns the selected prompt.
The native frontend tracks pending child operations, drains owned workers before
stdio close and provides the [child inspector](../frontend/conversations.md#managed-research-inspection).
`_harness/conversation/view` accepts `{sessionId}` and returns ordinary safe history
without recovery or a second controller for active-child viewing. A missing
terminal receipt remains null even when an interrupted execution is Unknown;
this does not fabricate a report or prove no effects occurred. Durable ownership-scoped Stop is distinct from a successful child read;
typed child input, scope approval and later-run continuation use the controls
above; complete restart qualification remains a separate obligation. Closing Harness does not authorize execution while
closed or automatic inference after reopening. Preserve held/unknown work for
explicit inspection and the appropriate Resume gate.
