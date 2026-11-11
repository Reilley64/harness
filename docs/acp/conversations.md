# ACP conversations and controls

## Repository-agent extension

`harness/agents: true` advertises the narrow durable Setup/Griller controls.
Read/start accept optional `role: "setup" | "griller"`; absent role uses repository
presence detection. See [repository Setup](../repository-setup.md) for its real
read/confirmation extension, native draft inspector and recovery contract.
The controls are:

- `_harness/agent/read` accepts `{cwd}` and returns
  `{conversation: null | <safe conversation>}`. A missing Griller is normal.
- `_harness/agent/start` accepts `{cwd, selection: {"harness/modelSelection":
  {providerId, modelId, effort}}}` and returns an existing or newly created Griller,
  without inference. Existing context wins over the supplied creation selection.
- `_harness/agent/resume` accepts `{sessionId}` after read/load/start and returns a
  standard-shaped prompt response with safe conversation metadata. It resumes the
  same interrupted run/reply through the ordinary coordinator/streaming updates,
  without a fabricated user message.
- `_harness/agent/acknowledge` accepts `{sessionId}` and returns `{conversation}`.
  It explicitly releases the interruption gate for a new human inspection
  instruction, without inference or altering recorded uncertain effects.

Safe conversation metadata adds nullable `agent: {id, role: "griller",
creatorId: null}`, `runs: [{id, agentId, replyId, task: "grilling", modelId,
effort, status}]`, and `resumeRequired`. Runs expose running, needs_human, stopped,
failed and interrupted states; provider call identities/continuation stay private.
Legacy records have no agent/runs rather than inferred factory lineage.

Standard `session/new/load/prompt/cancel` remain supported. Ordinary prompts cannot
bypass an interruption gate. Cancellation explicitly stops the active leaf and
never generates a handoff. Unknown outcomes block direct Resume. All controls use
closure-backed application use cases and bounded atomic conversation storage.
These agent routes do not implement child delegation or a generic factory-command
protocol. Separate [approved-work routes](../approved-work.md) provide exact-version
human approval, durable queries and separate CoS handoff; later execution/merge
capabilities remain unavailable.

## Approved-work extension

`harness/approvedWork: true` advertises `_harness/work/read` and the trusted local
`_harness/work/approve` operation. See [approved work](../approved-work.md) for exact
schemas, native approval, role metadata, persistence/admission and recovery. The
approval method is never an advertised model tool; ordinary chat/status cannot
authorize it. A delivered CoS uses the ordinary session/prompt/Resume/tool paths.

## Native Decider extension

`harness/nativeDecider: true` advertises the separate native-release preferences,
CoS assignment decision queries/select/Stop operations described in
[native Decider](../native-decider.md). Child dispatch uses its separate
[coordination extension](children.md), not native selection. Selection
requests have no application or SDK deadline and remain explicitly stoppable. Selection projections retain the complete candidate/evidence snapshot and all bounded hierarchy stages, encoded inputs, opaque memberships, outcomes and available per-stage native usage/cost/provenance. `_harness/decision/evidence/refresh` accepts `{id}` and returns `{checked_at, refresh_failed, evidence}` without inference or changing the accepted snapshot. It refreshes the same Harness-owned source, not a vendor inventory; retrieved data may still be stale/unknown by its original measurement date.

## Guidance extension

`harness/guidance: true` advertises read/adopt operations through real application
use cases. See [role guidance](../role-guidance.md) for request/response schemas,
resolution/override precedence, snapshots and recoverable blocks. Inspection
includes actual instruction text/source/version/upstream, not a synthetic agent
roster. Safe conversation metadata adds `selectedInstructions` and each run's
nullable `instructions` snapshot ID; `task` reflects its captured phase.

## Conversations

Standard ACP v1 methods present the conversation use cases:

- `session/new` calls `create_conversation` for unassociated pilot sessions.
  Native Griller creation uses the repository-agent extension instead. Its `_meta["harness/modelSelection"]` is required:
  `{ "providerId": "openai", "modelId": "<catalog-id>", "effort": "medium" }`.
  Effort may be null. The backend resolves the saved provider connection; no
  credentials or tokens are supplied by the frontend.
- `session/load` calls `read_conversation` with an explicit canonical UUID.
  It replays ordered user/agent chunks and tool calls at their persisted UTF-8
  text offsets, retaining stable message/tool IDs and Harness status metadata.
  Tool replay includes an initial `tool_call` and current `tool_call_update`.
  It returns authoritative conversation metadata, including partial statuses,
  without inference or credentials/catalog access.
- `session/prompt` accepts baseline text and `resource_link` blocks and calls
  `create_message`. Resource links become JSON-encoded user-content references
  marked as unread; they never trigger fetching or imply access to their contents.
  Optional image, audio and embedded-resource blocks remain unsupported. Mixed
  text retains its whitespace and the combined message has a 1 MiB UTF-8 limit.
  Optional model
  selection metadata changes the next reply's provider/model/effort after current
  admission and resolution of that provider's saved connection. The user message and in-progress assistant are stored first.
- `session/update` carries `agent_message_chunk` text with a stable `messageId`
  during inference. Credentials and opaque reasoning events are never forwarded.
- `session/cancel` signals the active reply. The prompt finishes only after the
  cancelled partial assistant is saved, then returns `stopReason: "cancelled"`.
  JSON-RPC request cancellation also signals rather than dropping message creation.

New/load/prompt responses include `_meta["harness/conversation"]`: ID, provider
connection ID, model/effort, ordered messages with stable IDs/roles/content/status
and per-reply provider/model/effort, timestamps, and revision. Message and run
snapshots add `providerId`; legacy single-provider records derive it only from
their stored provider. This is a safe presentation
snapshot, never the credential record. Incomplete and refused replies map to
`max_tokens` and `refusal`; normal completion maps to `end_turn`.

Sessions must be created, loaded, or opened through agent read/start in this ACP
process before prompting. Prompt
overlap is rejected locally and by the cross-process conversation lock.
Conversation errors use `-32004` with message `Conversation request failed` and
diagnostic text in `data`; malformed selection/content uses `-32602`. Generation
failure still saves partial text as failed. The frontend reloads the authoritative
snapshot after such errors rather than claiming an optimistic message was saved.

See [conversations](../conversations.md) for domain rules and durability limits.

## Token usage and workspace

Conversation metadata includes the canonical bound `workspace` and a `usage`
summary with nullable `inputTokens`, `outputTokens`, `contextTokens`, `modelId`,
and a boolean `partial`. Input/output are cumulative reported session tokens;
context is the latest request's input plus output, not the cumulative total.
Missing reports remain unknown; partial totals are lower bounds.

The `harness/usage` capability advertises `_harness/usage` notifications:
`{sessionId, usage}`. Reports are emitted after durable generation accounting;
retained model steps are checkpointed before their updates, including steps that
request tools. Discarded or cancelled generations contribute any reported counts
without contributing continuation. Final/resumed snapshots are authoritative.
Model metadata includes nullable `contextWindow`, sourced from the catalog's
optional `context_window`. No account quota or inferred token count is exposed.

The ACP projection also emits standard `usage_update` with `used` and `size` when
the latest reported context and matching model capacity are known. Its
`_meta["harness/usage"]` retains cumulative/partial details. Unknown counts or
capacity suppress that standard update rather than fabricating required values.
A model switch can defer standard usage until the final snapshot supplies its
capacity. Live updates and replay share the same projection implementation; the
custom notification remains available to existing clients.

## Tools and steering

Standard `tool_call` and `tool_call_update` updates include
`_meta["harness/tool"]` with the stable execution/message IDs, original argument
JSON, exact Harness status, preview, artifact path, truncation flag, elapsed
milliseconds and UTF-8 text offset. Terminal conversation snapshots contain the
same `tools` summaries and accepted `steering`; provider continuation is private.

`_harness/conversation/steer` accepts `{sessionId, id, text}`, where `id` is a
canonical UUID. Its `{accepted:boolean}` response is acknowledged only after
persistence, and accepted IDs are deduplicated conversation-wide. An identical
UUID/text retry is acknowledged without mutation or generation restart, including
after its original turn; a UUID bound to different text is rejected. Idle retries
check persisted history using the same identity rule. The same turn continues; clients
must not implement steering by cancelling and resending the original request.
Budget exhaustion finishes with a tool-free summary, message status `limited`,
and standard stop reason `max_turn_requests`. Standard `session/cancel` cancels
without a follow-up. All enabled tools run
automatically after any required MCP connection trust: there are no per-call
permission requests. See [tool calls](../tool-calls.md) and
[MCP connections](../mcp-connections.md).
