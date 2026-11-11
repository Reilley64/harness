# Provider waits and usage

## Worker outcomes and admission

The ordinary model-step adapter classifies actual Responses HTTP outcomes for the
selected provider, opaque connection and model. A wait belongs to the same saved
reply and durable agent; it is not a slot reservation, account-capacity estimate,
worker substitution or a sent inference. No repository, role or fleet semaphore
is introduced. A rejection does not establish capacity for another connection or
model, and Harness does not query credit/reset endpoints or poll models.

Authentication/request/context/budget failures are permanent. HTTP 408, 429 and
5xx, supported rate-limit/overload codes, connection failures and pre-output stream
interruptions are transient unless structured metadata identifies depleted quota
or a permanent cause. Authentication/request HTTP statuses take precedence over
conflicting error metadata. Once rejection headers establish a status, its
classification and valid timing survive an interrupted or timed-out error-body
read; only a timeout without a known outcome supplies a generic transient failure. `insufficient_quota`, `quota_exceeded`,
`insufficient_credits` and `credits_exhausted` identify quota. OpenRouter's numeric
402 with `Insufficient credits.` identifies depletion; an unaffordable request,
fewer-max-tokens condition or otherwise unknown 402 is not an unchanged retry
loop. Arbitrary provider prose is not a retry-allowance authorization.

Valid Retry-After delta seconds or HTTP dates are retained as absolute Unix
milliseconds, within the representable non-stale range through year 9999.
Overflow, stale dates and out-of-range values are unusable metadata, not an
invented future reset. Missing/invalid transient timing uses exponential equal jitter:
1–2 seconds after the first failure, 2–4 after the second, doubling to a bounded
30–60-second range for explicitly extended work. Quota without usable timing has
no invented reset or ETA and waits for explicit human Resume. Known reset timing
permits continuation only while the existing controller remains eligible.

The default is at most **two automatic transient retries after the initial
failure**. Trusted approval can explicitly configure an initial allowance of
0–64 for its exact approved work; omission and older approvals retain default two.
Zero disables automatic transient retries, not the initial request or explicit
quota Resume. The immutable approval intent/version binds this choice alongside
the model pair; changing it requires a deliberately new approval intent, not prose.
CoS and its authorized Research executions initialize new episodes from that
recorded policy. Existing episodes retain their original counts/allowances even
if current work changes, and ordinary admission still gates any continuation.
Post-exhaustion extensions remain separate human operations. See
[approved work](approved-work.md#trusted-approval-and-handoff) and the
[approval UI](frontend/configuration.md#approved-work-and-cos).

The allowance and failure count are checkpointed with the exact reply,
selected pair and its existing assignment/approved-work association. Steering,
model changes, a later run, reconnecting or reopening cannot clear an outstanding
wait or reset its allowance. A successful model step retires that retry episode
into retained history. Existing model-step/tool budgets still apply. The HTTP
client explicitly disables Reqwest retries; no SDK/gateway retry layer amplifies
the application allowance. Output already delivered is not automatically resent
after a stream failure.

## Live waits, recovery and human controls

The controller suspends before any new generation reservation. It retains its
logical run, accepts ordered steering and observes the existing ownership,
Stop/input/approval/restart fences. Suspended provider-wait time does not consume
the active-turn deadline. Waiting is neither cancellation/resend nor a native
Decider deadline. Independent children can continue; their results remain durable
until a safe eligible continuation boundary.

Unknown-reset quota and exhausted transient waits leave the live controller
suspended, not falsely crashed. A trusted wait-control update only changes the
checkpointed retry authority with a checked conversation revision; it does not
start another controller or clear any ownership/recovery gate. The controller
reloads that checkpoint before proceeding, then revalidates current model,
connection, policy, assignment and generation eligibility through ordinary
admission. A conflicting write fails rather than silently replacing a newer fact. All
conversation writers share a separate per-record mutation lock covering the
revision check and complete replacement, including live human controls and
controller checkpoints. Contention rejects the update for fresh inspection; it
does not queue inference, overwrite a newer snapshot or acquire the controller's
long-lived execution lock. The short mutation lock is never held over model or
network execution.

After real process loss, ordinary conversation recovery marks the unfinished run
Interrupted and requires explicit Resume, including when a saved retry time has
already passed. Loading/inspection never starts inference. Stop still cancels the
actual execution subtree; a timer or retry-control update cannot reopen it.
Completed/uncertain tool effects and reported cancelled/superseded usage remain.

Native controls:

- The footer distinguishes timed retry/quota waits, unknown-reset Resume and
  exhausted allowance. `/usage` reads accounting without inference, including
  during a live wait. `/usage run <run-id>` and `/usage spec <work-id>` narrow it.
- `/resume` authorizes an unknown-reset live quota wait, or uses ordinary explicit
  recovery Resume when no live controller exists. A transient exhaustion requires
  `/retry extend <n>` (1–64), not natural-language steering. An extension applies
  only to the retained episode; it cannot reset its count, shorten a retained
  valid Retry-After/reset time, or select another model.
- `/retry abandon` is an explicit idle control after Stop/inspection. It retains
  the old allowance and attempts in history, starts no inference and clears no
  independent recovery/approval gate. New ordinary prompts cannot abandon waits.

Public application operations are `read_usage` and `update_provider_retry`, in
addition to ordinary prompt, inspect, Stop and Resume. ACP exposes
`_harness/usage/read` with exactly one of `sessionId` or `workId`, optional `runId`
only with a session; `_harness/provider-retry/update` takes `sessionId`, inspected
`revision`, a fresh human control `id`, and `additional`: positive for extension,
zero for quota Resume, null for idle abandonment. These are trusted-client
controls, not model tools. No control grants scope approval, reconciliation or
permission to replay uncertain mutations.

## Durable accounting

Conversation format eight explicitly retains retry authority/history and a
parallel generation-accounting ledger. Readers continue accepting versions one
through seven without inventing reports or approvals. Older executables reject
eight. New accounting records retain the actual connection at reservation time,
reported tokens/cost independently of continuation, outcome and an optional static
catalog USD estimate. A confirmed unsent reservation is removed; attempted,
cancelled, superseded and crash-uncertain reservations remain. Missing reports are
unknown, not zero. Safe reports do not restore discarded private continuation.

Usage reads enumerate bounded source IDs, then use the ordinary validated
conversation/decision codecs without recovery or writes. Worker identities are
conversation/reply/generation index; native identities are intent/stage. Repeated
reads do not append or count a report twice. Spec membership follows recorded
assignment ownership, never role names or transcript prose. The query covers any
actual coordinator/Research/worker records and can project future role records;
it does not fabricate unavailable review/Implementer executions.

Schema-two native top-level responses are not additional billable stages. Planned
or stopped-before-admission stages do not count as sent work; admission without
a report is explicitly reserved-or-sent unknown. Legacy requests have no invented
stage history. Newly prepared native intents retain their actual parent run when
available. Older records without it remain unassigned in run-level queries and
are counted explicitly as `unassignedNativeRuns`; agent/spec totals still include
their available reports. Inert legacy reads and safe Stop/late reports remain
schema one under policy A. No migration, native retry, replay, hidden deadline,
fallback or pool/evidence validation change is authorized by accounting.

Amounts are separate: token reports, reported API USD, static catalog estimates,
unknown API costs, and subscription-allowance attempts. Subscription usage is not
incremental API dollars. Static estimates use captured input/output list prices,
not current billing, cached-token knowledge or subscription marginal prices.
Reported totals with unknown entries are lower bounds, not complete spend.
Source revisions, captured list-price snapshots and per-entry provenance remain
available in the public query; unavailable pricing retrieval/effective dates stay
null. Historical messages with unprovable generation gaps or missing source
conversations are listed in `unknownHistorySources`, separately from actual
attempt/stage entries. They cannot turn an empty retained report set into a claim
of zero usage. Per-record reads are of evolving storage, not a transactionally
locked invoice. Ownership metadata is projected separately and full histories are
processed one at a time; unrelated private transcripts are not retained together
for the aggregate. A work query needs a retained assignment/intent rather than
inventing an empty scope. An unrepresentable currency sum blocks visibly instead
of becoming a null/zero amount; retained individual reports remain authoritative.
There is no whole-spec spend/time cap or billing-completeness claim.

Source enumeration is bounded at 4,096 records per family and projection at
100,000 entries. Retry histories retain at most 1,024 episodes, 64 explicit
extensions per episode, and the existing generation bounds. These are retained
representation limits, not concurrency slots: exceeding required capacity blocks
visibly instead of deleting authority or silently omitting usage.

## Verification boundary

Temporary external probes exercise real stock ACP/HTTP/storage paths with
labelled loopback PKCE/catalog/worker responses, default retries, timing,
contextual 402, exhaustion/extension, steering, scope independence, Stop and a
real SIGKILL/restart. Native launcher byte probes and current source/results are
retained outside the checkout. Their exact revisions/results and missing cases
are owned by the implementation evidence packet, not inferred from compilation.
Scripted provider outcomes are not live entitlement, actual billing, native-model
quality, human visual acceptance or whole-MVP acceptance. Later review/workspace
runtimes, live authentication/paid exercise and target merge retain separate gates.
