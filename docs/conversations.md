# Conversations

## Scope and terms

A conversation has a stable UUID, a provider-connection reference, a current
model/effort, ordered messages, timestamps, and a revision. A message has its own
stable UUID, user/assistant role, text, status, and model/effort provenance. Message
roles are separate from agent roles such as griller or researcher.

Messages are UTF-8 text with durable tool steps and accepted steering instructions.
Native tools, proposal capture and shared trusted stdio/Streamable HTTP MCP use
the same tool loop.
A separate approved-work CoS and managed Research use that loop. Attachments and
a general conversation picker remain unavailable. See [approved work](approved-work.md). OAuth tokens stay in Rust and are not proactively
injected into messages or ACP; explicit tool output may contain readable secrets.
See [tool calls](tool-calls.md) for the execution and permissions contract.

## Workflow

1. Startup uses the [repository Setup presence detector](repository-setup.md) to
   select Setup for missing outputs, otherwise Griller, and reads its saved
   conversation without inference. An absent actor renders an empty view and
   creates no conversation file.
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

## Managed Research and parent work

[Managed dispatch](managed-dispatch.md) owns Research execution claims, joins,
receipts, safe-boundary delivery and required follow-up. A model response alone
is not parent settlement. [The ACP reference](acp/children.md) owns its public
wire shapes.

## Provider requests

See [worker requests, private continuation and accounting](conversations/providers.md).

## Persistence and crash recovery

See [record versions, atomic replacement and conservative recovery](conversations/recovery.md#persistence-and-crash-recovery).

## Catalog caches

See [catalog configuration and caching](catalog-configuration.md#catalog-caches).

## Explicit resume

See [explicit history loading](conversations/recovery.md#explicit-resume) and
[interrupted execution](conversations/recovery.md#repository-griller-and-interrupted-runs).

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

See [durable identity and interrupted-run controls](conversations/recovery.md#repository-griller-and-interrupted-runs).

## Fresh-session model preference

See [selection persistence](conversations/recovery.md#fresh-session-model-preference).

## Validation

See [verification boundaries](conversations/recovery.md#validation).
