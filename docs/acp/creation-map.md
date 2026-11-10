# Creation-map inventory

`initialize._meta["harness/creationMap"]` advertises `_harness/agents/map`.
This is a read-only metadata operation, not a conversation list, agent factory,
execution tree or authority to run work.

## Request and pagination

```json
{
  "method": "_harness/agents/map",
  "params": {"cwd": "/repository", "after": null}
}
```

`cwd` is resolved to the repository identity. `after` is absent/null for the first
page, otherwise the opaque canonical UUID cursor from `next`. Unknown request
fields and malformed/noncanonical cursors are rejected. The cursor orders stored
ConversationIds exclusively; it is not an AgentId or a run identifier.

Each page selects at most 64 inventory entries after the cursor and returns
at most 64 agents. Directory discovery scans metadata; owner-consistency checks
also read referenced child sources. `next` is the last selected inventory entry when another page
exists, otherwise null. Filtering unassociated/foreign-workspace conversations
can make a page shorter, so continue using `next`, not the returned length. There
is no 64-agent fleet cap. Readers refresh from the first page to observe additions
that sort before an earlier cursor; pagination is not a transactional global
snapshot.

## Response

`agents` contains the following metadata for associated agents in the requested
repository:

| Field | Meaning |
| --- | --- |
| `id` | Durable AgentId. A later run is not another node. |
| `conversationId` | Address of the recorded conversation to inspect/open. It need not equal AgentId. |
| `creatorId` | Recorded creator AgentId from Research/approved-work association, or null. Never inferred from role or ownership. |
| `role` | Recorded domain role, not a static roster entry. |
| `revision`, `createdAt` | Saved conversation revision and creation time. |
| `savedStatus`, `runId` | Latest recorded run facts, nullable; not a process-liveness assertion. |
| `resumeRequired` | Retained conversation recovery flag. |
| `coordination` | Nullable own status/gate, authority-completeness, restart flag and revision. |
| `execution` | Nullable retained owning Research execution, using the existing [safe child projection](children.md). |
| `pendingResults` | Owned-child receipts not yet delivered to an eligible parent boundary. |
| `requiredFollowUp` | Owned-child receipts without completed parent follow-up. |

The projection deliberately omits messages, task/prompt bodies, tool content,
credentials and provider-private continuations. Instruction references in child
metadata are hashes, not instruction bodies. Completed/stopped agents remain in
the inventory. Ad-hoc legacy conversations without associated agent metadata are
not assigned invented identities or creators.

A Research node's recorded assignment/execution identities must agree with its
retained owner. An owner's known children must still have their required sources
and matching ownership identities. Missing/incomplete ownership or missing/corrupt
required sources fails closed, rather than returning a fresh healthy inventory
that silently omits a known child. A frontend can retain its last known layout
with an explicit stale/unavailable warning while retrying metadata reads.

## Authority boundary

Creation arrows do not grant ownership or determine Stop scope. For example, an
approved-work CoS can have a Griller creator while owning a separate execution
root containing Research A and B. Stopping the Griller does not traverse that
creator arrow. [Managed dispatch](../managed-dispatch.md) owns execution policy,
scoped Stop, capture membership, receipts, recovery and follow-up.

Reading this inventory performs no recovery, inference, dispatch, input answer,
approval or settlement. It does not replay effects or create a controller.
Opening an existing node uses the read-only conversation view. Actual interaction
still uses the existing backend interfaces and their admission gates.

A completed child assignment may coexist with its durable agent's generic
needs-human next-turn gate. Display the terminal owning-execution outcome without
turning a completed assignment into an unanswered input request. Unknown authority
and restart requirements remain blocking. Neither child completion nor human or
model prose settles the parent's approved work.
