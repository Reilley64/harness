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
| `_harness/children/join` | `{sessionId, id, mode, executions}` | Capture a fixed `any`/`all` set on the parent's active logical run. |
| `_harness/children/background` | `{sessionId, id}` | Release that captured join without restarting children. |
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

## Responses

All operations return `{coordination, execution, result}`, using null for fields
not supplied by that operation. Missing coordination or receipt is null, not a
fabricated child, successful completion or zero-work assertion. Corrupt storage
fails through the normal safe conversation error boundary (`-32004`); malformed
requests use `-32602`.

`execution` has `intent`, `assignmentId`, `agentId`, `executionId`, `creatorId`,
`parentRunId`, `parentRevision`, nullable `workId`/`workVersion`, `role`, `providerId`,
`modelId`, nullable `effort`, `instructions`, `routed`, `status`, nullable
`outcomeId`, `deliveredTo`, `followUpCompletedBy`, and a `source` conversation URI.
Status is `held`, `running`, `completed`, `needs_human`, `failed`, `stopped` or
`unknown`. Creation lineage is separate from actual Stop ownership.

`coordination` has `agentId`, `revision`, `gate`, nullable `runId`, `settled`,
`children` (execution summaries), and `joins`. A join has `id`, `runId`,
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
actual `creatorId`, and nullable `research: {parentId, intent, assignmentId,
executionId}`. Research run task is `research`. Existing schema-six Setup/CoS
metadata and private-state exclusion remain intact.

## Admission, cancellation and shutdown

Application use cases own admission, immutable selection, revisioned claims,
result delivery and joins; handlers translate requests and run connection-owned
operations. Reads perform no inference or tool execution. Dispatch checks its
actual parent/run and exact work authority rather than trusting supplied role
text. Scope approval remains the separate trusted approved-work operation.

Cancelling a pending dispatch signals its cancellation token and awaits the
owned operation; an admitted write is not promised to roll back. Once an accepted
handle is returned, cancelling that response wait is not a child Stop operation.
Join/background currently await their atomic storage operation; cancellation is
not an inverse mutation. Lost responses require read/reconciliation with the
original keys, not a new intent or blind effect replay.

The ordinary session cancellation/shutdown path still owns the selected prompt.
The native frontend has not yet added child-operation draining, status controls
or active-child navigation. Cross-process owned-subtree cancellation and complete
restart wake/reconciliation remain follow-up integration, not capabilities implied
by a successful child read. Closing Harness does not authorize execution while
closed or automatic inference after reopening. Preserve held/unknown work for
explicit inspection and the appropriate Resume gate.
