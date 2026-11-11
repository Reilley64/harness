# Native Decider selection and inspection

## Implemented boundary

An actual approved Chief of Staff (CoS) can prepare a durable assignment intent,
then a trusted native user can choose and inspect a typed native decision for it.
This slice **does not launch a child**, create an agent/run/worktree, dispatch
review, or integrate/merge anything. A selected pair is not executed work.
[Managed Research dispatch](managed-dispatch.md) owns subsequent execution. Complete-pool hierarchy and sourced evidence are part of selection; hosted routers remain disabled.

## Prepare through CoS

The normal shared tool loop advertises `prepare_assignment` with:

```json
{
  "intent": "<canonical UUID for this new assignment>",
  "role": "implementer",
  "task": "<bounded assignment>",
  "constraints": "<bounded constraints>",
  "toolContext": "<required normal Bash/file tool context>",
  "contextTokens": 4096
}
```

Roles are `research`, `implementer` or `reviewer`. Task is at most 4,096 UTF-8
bytes; constraints and tool context are each at most 2,048. Context is a required
capacity, not a measured/predicted token count; it must be 1..1,000,000,000.
Do not deliberately include credentials or full transcripts. Task/repository
content is untrusted data and cannot broaden policy, supply executable identities,
change the selected native Decider or approve work.

Preparation validates the actual CoS identity and its exact approved handoff
against current work authority. It captures the actual parent's model **and
thinking**, connection, agent/session/revision and work/version before any native
request. It persists all brief fields, without summarizing or truncating them.
Repeated same UUID/CoS/work-version/brief reuses the accepted snapshot and result,
including after the parent changes models. Changed brief/role/authority under that
key conflicts; a deliberate new key is new intent. No fuzzy deduplication occurs.
Existing approval, assignment identity and conversation schema-six authority are
unchanged. Griller/legacy conversations cannot prepare managed assignment intents.

## Native picker and inspector

- `/decider` opens a separate native-release picker. Arrows highlight, Enter saves
  the user-local selection, Escape dismisses. Moving aliases are labelled. There
  is no native thinking control, compiled default release, automatic allowlist
  edit or paid connect/selection probe.
- Choices come only from the existing Harness-owned catalog/cache at `MODELS__URL`.
  The static catalog is not imported, regenerated or merged with a second provider
  catalog. A positive `decisions` capability, supported OpenRouter typed transport,
  bounded context, no native effort and non-router identity are required. Workers
  still independently require positive `chat` plus their existing protocol checks.
- `/decision <intent>` reads the durable inspector without inference. It shows the
  complete brief, captured parent/authority, policy and exact candidate snapshots,
  requested/resolved native identity, optional serving provider, reported usage,
  nullable cost/confidence, selected pair and recovery reason. Missing values are
  unknown, not zero. `s` selects once for a prepared intent. Repeating or reopening
  never retries a pending/settled request or pretends that a child ran.
- `x`, then `y` explicitly Stops native work; `n` declines. Escape only navigates
  and never cancels a pending request. `o` opens the actual parent CoS for free-form
  discussion, visibly under its normal CoS title and chat path. Native Decider does
  not chat or call tools. Per-conversation drafts/viewport remain independent.
- Configuration-required results preserve the intent/selected pair and open the
  existing model-settings surface with a reason. `/auth` remains explicit. Cancel
  preserves the block without selecting a replacement or reopening login loops.
  Before admission, missing native preference, authentication, allowlist and
  capability failures expose distinct safe configuration reasons without changing
  the prepared intent, starting inference or selecting a replacement.

In-flight UI ownership is keyed by intent and connection, not a panel-wide busy
flag. Inspecting or selecting another prepared intent does not relabel it as the
first pending request. A completion updates the same reopened inspector, while
revision guards prevent older reads or delayed Stop acknowledgements from replacing
a newer result, including late stopped accounting. Escape and actor navigation do
not cancel outstanding requests.

## Policy, eligibility and validation

`harness-suitability-cost-v1` prioritizes adequate task suitability, then
cost-conscious choices among adequate options. It does not request the strongest
benchmark blindly, the cheapest regardless of adequacy, a universal weighted score,
a correctness threshold, or a proprietary optimizer. Code validates required
context capacity, supported thinking, actual connection, current positive allowlist
and implemented chat/tools/continuation. Missing benchmark or price evidence does
not exclude an otherwise eligible pair. Each opaque UUID key maps to exactly one
connection/model/thinking pair, with separate explicit `abstain`.

The complete candidate snapshot is immutable and contains every currently eligible connection/model/thinking pair once, including omitted/default thinking where the existing worker validator permits it and every explicit supported effort. Default thinking is not imputed as a benchmark's explicit effort. Code prepares complete bounded
batches, retaining opaque membership keys and the exact encoded input for every
stage. Each native request has at most 254 pairs plus explicit abstention (255
options), a 24 KiB encoded JSON envelope, and the native model's context number
as a further byte ceiling, not a tokenizer measurement. The full brief, policy,
options, abstention and evidence all count toward the envelope. A candidate that
cannot fit alone fails explicitly without truncation or inference.

All required batch winners advance through as many reducing hierarchy levels as
needed. Every level uses the same versioned suitability/cost policy and immutable
candidate evidence; neither response order nor an option's array position grants
identity. Distinct stage question IDs and opaque candidate keys are validated.
Before each serial native effect, the exact stage's admission timestamp is checkpointed. Inspection distinguishes planned, pending, completed, failed and stopped/unknown stages; this timestamp is an application reservation, not a provider billing/HTTP receipt. Refusal and explicit abstention remain distinct recorded failure outcomes. A required request, validation, refusal or abstention failure fails routing: it
cannot exclude that batch and promote surviving winners. Only one successful
final stage can select a worker. This hierarchy makes no global-optimality claim.
A non-reducing level that cannot compare two complete candidates is Unsupported,
not permission to omit candidates. No native application deadline or retry is added.

## Sourced evidence and refresh

Selection reads routing evidence from the **same Harness-owned** `MODELS__URL`.
The source/cache adapter never imports another vendor inventory, widens the
curated catalog, changes capabilities or grants entitlement. The original 172
catalog rows remain unchanged. Optional per-row `routing_evidence` observations
are advisory and bind to provider/model/canonical identity and exact effort;
incompatible observations are not borrowed. The catalog's existing static list
prices remain labelled static USD per million tokens, not current billing or
subscription incremental dollars. Missing prices/benchmarks remain unknown, not
zero or automatic exclusion.

`APP_DATA_PATH/routing-evidence.json` is a separately versioned, bounded private
cache. A selection read refetches after 24 hours; there is no background poll.
Catalog, capability and pricing facts use a 24-hour freshness window; benchmarks
use seven days from their original measurement, as well as retrieval age. A
24-hour source fetch may retrieve benchmark data too; it never resets measurement
age. Explicit `_harness/decision/evidence/refresh` with `{id}` refetches on demand,
without native inference, replay or rewriting an accepted decision snapshot.
It returns current cache observations and a truthful refresh-failed indicator.
Prepared intents without a pool still refresh the source, returning no pair facts.

Each observation retains category, metric/value/units, attribution and source URL,
revision, retrieval, measurement, publication/effective dates, requested/resolved
model and serving provider, effort and configuration. Code owns dates, finite
numbers and units; the native model does not calculate them. A refetch of an old
measurement remains stale. Unknown original dates remain unknown even after a
successful fetch. Missing values and incompatible efforts receive explicit
unknown observations; no new benchmark score or measured fact is fabricated.
Source failures persist their failed-refresh state across cache reads/restart, retaining original retrieval/measurement dates with explicit stale status. Without a successful prior retrieval, retrieval/revision/value fields remain null/unknown, never fabricated zero/current values. Original source failures are retained internally, including alongside a cache-commit failure.
Unsafe/corrupt/unsupported cache records fail rather than being overwritten as
healthy. Sourced observations never replace live auth/allowlist/protocol/context,
tools/thinking or canonical checks, including final worker admission.

The optional source schema is documented in
[catalog configuration](catalog-configuration.md#routing-evidence-observations).
Inspection exposes the original complete pool, captured facts/ages, each input,
stage membership, successful/failed/pending outcome and all available per-stage
usage, cost and native serving provenance. Missing reports/costs stay unknown;
the top-level response is not an additional billable stage. The ordinary
[usage query](provider-waits.md#durable-accounting) projects those exact stage
identities without rewriting records, retrying or inferring missing historical
parent runs. Newly prepared intents capture the actual parent run where available;
policy-A legacy records acquire no invented run/stage provenance. Captured evidence
ages describe the snapshot, not a claim that an old decision is fresh today.

The response must contain exactly the requested native `choice` or `refusal`
answer. Required envelope/usage types, membership, finite/ranged supplied confidence
and probabilities, complete normalized supplied probability distributions,
nonnegative finite optional cost and bounded token counts are validated. Optional
confidence is not calibrated correctness probability and has no cutoff. Explicit
release responses must match the requested or catalog canonical identity; moving
aliases retain the resolved identity. A router cannot become a native selection.
The selected worker is revalidated against current auth, allowlist, canonical
revision, context and effort before acceptance and on subsequent inspection/use.
No explanation pretends to be a native reasoning trace.

## Failure, Stop and recovery

Genuine request/validation/refusal/abstention/stale-choice failure can select only
the **exact captured parent connection/model/thinking**, if still eligible. It
never lowers effort or uses a coded default. An ineligible parent produces a
configuration block with no replacement. A user disallowing an already selected
native model or returned worker instead produces configuration recovery, not
parent replacement. Changing a settled worker's policy similarly preserves the
record and exposes a current block. A denial at the final worker revalidation is
classified from its typed policy cause, not mistaken for generic stale-choice
failure and parent replacement. Ordinary worker failures do not invoke this
selection path or authorize model switching.

The native adapter uses its own Reqwest client with no request/read/connect
application deadline, no inherited worker timeout, no SDK retry, redirects disabled,
and `provider: {allow_fallbacks:false, require_parameters:true}`. It sends no tools,
reasoning effort, hosted advisor, fallback-model list or chat request. Pending work
ends only on completion, actual failure or explicit Stop. The application observes
its own durable Stop control while a request is pending (including another local
process); this is local record observation, not provider/model polling or a
scheduler. Unrelated worker/auth security limits remain unchanged.

Requests use `DECIDER__URL` (default OpenRouter `/api/alpha/decisions`) and the
existing protected `auth/openrouter.json` credential path. Origin must match both
the configured OpenRouter issuer and saved credential binding. HTTPS or explicit
loopback HTTP fixtures are supported; URL credentials/redirects are rejected. Keys
never cross ACP or enter the decision body. No authenticated/paid service exercise
is implied by implementation or local scripted verification.

Records under `APP_DATA_PATH/decisions/<intent>.json` use an independent schema-two representation bounded at 64 MiB, owner-only no-follow reads/locks, revision-checked atomic replacement and directory sync. Under human-approved policy A, existing schema-one records remain schema one, including safe explicit Stop/accounting and late-report updates. Their identities, captured parent/brief/authority and provenance remain intact; absent historical stages/evidence remain absent/unknown. Legacy Prepared selection visibly blocks and requires a deliberately new CoS intent/UUID through normal authorized preparation. No conversion, automatic re-preparation/new UUID, native attempt, retry/replay or parent-fallback bypass occurs. Legacy Pending/settled/Stopped reads remain inert with current eligibility checks and no native resend. Only new intents use schema two, with explicit validated pool/stage lifecycle states and complete per-candidate category observations, including explicit unknowns. Missing or empty required new facts fail closed. No same-identity migration, downgrade, migration API/background conversion or deletion is authorized. Unsupported versions and inconsistent stage/input/snapshot relationships fail closed. Short process locks cover claims/checkpoints,
never native inference or a repository scheduler. The native request snapshot is
committed before HTTP, and outcome/report/finish time before return. Duplicates
reuse the accepted intent; conflicting transitions reject. Recovered pending work
is inspectable/unknown and never automatically re-sent; explicit Stop can settle
it. This slice has no Resume/retry/dispatch engine for an uncertain native request.
A new intent is an explicit new request, not a hidden retry. Stopped requests never
fall back. Stop marks every unfinished stage: admitted stages retain an unknown
outcome, while unadmitted stages explicitly record Stop before admission. Completed
stage history remains unchanged. Schema two rejects stopped records missing these
stage outcomes. Missing reports/costs remain unknown rather than fabricated zero usage.

A received valid accounting/provenance report is durably checkpointed before
post-response authority or configuration reads. While Pending or Stopped, its
`response` contains only tokens, optional cost and resolved/provider provenance:
choice and confidence are null and probabilities empty. This is accounting, not
a refusal or accepted selection. A competing Stop retains its outcome, finish
time and lack of selected worker. The store atomically enriches that exact stopped
request with a late report once; duplicates reuse it and conflicting reports fail.
An explicit Stop of the same still-unsettled immutable intent also incorporates
intervening accounting-only checkpoints rather than losing to their revision bump;
a settled selection cannot be replaced by Stop. No inference, replacement or replay
occurs to recover accounting. Network/body capture remains cancellable. Once the
bounded complete body is owned, blocking decoding/conversion is awaited outside
the cancellation race, so even a completed decode awaiting its executor handoff
cannot lose valid accounting. Invalid or missing selection-answer shapes retain
independently validated usage/provenance through the typed error port; they do not
become a valid choice or refusal. Invalid accounting itself remains unknown.

Cancellation of the owned select request is checked again after post-response
authority, allowlist, blocking validation and worker/parent eligibility awaits,
before accepting a choice or recovery. An already-cancelled request settles through
the same semantic Stop transition and atomic exact-intent arbitration, retaining
received accounting without choice, confidence or replacement. Genuine storage
errors still fail; cancellation is not inferred from an arbitrary save failure.

Recovery outputs, including successful blocked inspector/picker queries, preserve
internal original error chains until the presentation boundary. Worker-pool
eligibility failures retain their original sources independently, including when
other pairs remain eligible, the pool is empty, or a later operation fails. Error
results also own these diagnostics until disposal. Failed Pending-claim recovery
preserves both the original write cause and a subsequent read cause; neither
failure becomes successful admission. Enumeration allows at most
4,096 internal eligibility diagnostics; reaching that bound returns explicit
Unsupported without inference or a truncated pool. Persisted reasons and ACP
project bounded safe categories rather than raw transport bodies, credentials or
internal error strings. A storage failure is an error requiring inspection, not
a successful result with silently lost accounting.

## Public interfaces and evidence boundary

Application use cases are `create_assignment`, `read_decision`, `update_decision`,
`read_deciders`, `update_decider` and `update_routing_evidence`. Effect ports are `ReadDecision`,
`UpdateDecision`, `CreateNativeDecision`, `CreateDecisionInputs`, `ReadRoutingEvidence`,
`ReadDecider` and `UpdateDecider`, each with
its matching infrastructure adapter/constructor and composition entry. Domain
rules remain pure and Serde-free; each adapter owns independent DTOs.

ACP advertises `harness/nativeDecider: true`:

- `_harness/deciders`, `{}` → `{models, selected}`.
- `_harness/decider/update`, `{modelId}` → `{models: [], selected}`; trusted user
  preference only, no model tool or inference. Preference is
  `APP_DATA_PATH/preferences/decider.json`.
- `_harness/decision/read`, `{id}` → `{decision, blocked}`; missing is null.
- `_harness/decision/select`, `{id}` → the same durable projection and current
  block. It has no application/SDK timeout. Request cancellation explicitly Stops.
- `_harness/decision/evidence/refresh`, `{id}` → `{checked_at, refresh_failed, evidence}`; source/cache refresh only, with no inference or accepted-snapshot mutation.
- `_harness/decision/stop`, `{id}` → the stopped/settled projection. Terminal
  decisions cannot be replaced by Stop. No request accepts arbitrary response or
  candidate data, and there is no debug/fault route.

Temporary external probes retain real public orchestration/storage/ACP and native
HTTP adapters with scripted loopback OAuth/catalog/worker/native responses. They
cover exact parent capture, duplicate/conflict/concurrency, malformed/refusal/
abstention/numeric/stale/disabled/no-eligible recovery, oversized pools, real held
requests and Stop, actual process restart, native launcher controls and inherited
compatibility. This verifies the substituted boundary only: PTY bytes are not
human visual acceptance, native live quality, paid billing, assembled acceptance,
tracker completion or master integration. The live gates remain separate.

Primary development reference: [OpenRouter native Decisions contract](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request).
Published request/response claims informed the adapter; independently exercised
behavior here is limited to scripted loopback HTTP, not OpenRouter's live service.
