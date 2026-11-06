# OpenRouter, Jev, and evidence-backed model routing

Verified **2026-11-06**, public-source retrieval approximately **21:45–21:49 UTC**.
Research for [HARNESS-4](https://app.plane.so/reilley/browse/HARNESS-4/), within
[HARNESS-1's accepted boundaries](https://app.plane.so/reilley/browse/HARNESS-1/).
This is research, not an implementation specification or authorization to execute
additional models. No authenticated inference, paid calls, secret requests,
browser automation, product changes, or private-account inspection were used.

## Finding and scope

**Jev is identifiable and publicly listed as available: TypeSafe's System One
structured-decision model.** The current explicit OpenRouter model ID is
**`typesafe/jev-1.14`**, canonical revision **`typesafe/jev-1.14-20261014`**.
The moving alias **`~typesafe/jev-latest`** currently targets it. The creator calls
the underlying release **`jev-1.14.0`**; that is not the OpenRouter request ID.
`typesafe/jev-1.13` remains listed. Both creator documentation and OpenRouter's
live Decisions catalog and endpoint API agree on the identity. This is not a
substitution for the user's name. [S1–S5]

Jev is a plausible low-latency **Decider**, not a tool-executing coding model:
it takes state and typed questions and returns choices/probabilities, not prose,
code, tool calls, or reasoning traces. Its documented interfaces are
`POST /api/alpha/decisions` and the compatibility `POST /api/v1/systemone`, **not
Chat Completions**. Availability here means public catalog/provider presence
and documented access with an OpenRouter key, not a successful call by this
user or a measured Harness service-level objective. [S1–S7]

**Do not confuse it with `typesafe/jev-router`.** That separate hosted router
selects and executes another model. Its include lists are preferences that can
be relaxed, and its reasoning effort is a hint that can be changed. It is not
evidence that Harness can constrain a local Decider to exactly its enabled
executors. Using it would be a different product decision, not an equivalent
integration. [S8]

The evidence supports proceeding to an interface/design decision for **direct
Jev typed selection**, subject to an explicit release/alias policy, deterministic
candidate validation, and later authorized latency/quality/access verification.
It does not support an autonomous provider expansion or a universal benchmark
ranking.

## 1. Public verification and identity traps

Public GETs were made with `curl -sSL`/`curl -fLsS`, without an Authorization
header. JSON was inspected with Python; HTML was read as text using Python's
standard-library HTML parser. Sources are linked directly below; raw temporary
captures were kept outside the repository and are not a durable evidence bundle.
The relevant observations and response fingerprints are recorded here.

| Observation | Public response on verification date |
| --- | --- |
| `GET /api/v1/models` | 458 entries; defaults to **text** output, so Jev decisions are absent. Only `typesafe/jev-router` matched `jev` in ID/name. |
| `GET /api/v1/models?output_modalities=decisions` | 36 entries, `links.next: null`; includes Jev 1.14, Jev 1.13, and the latest alias. |
| `GET /api/v1/models?output_modalities=all` | 703 entries, `links.next: null`; includes those three plus Jev Router. |
| Jev latest catalog row | `alias_target.slug: typesafe/jev-1.14`; `output_modalities: ["decisions"]`. |
| Jev 1.14 endpoint lookup | One TypeSafe endpoint, named `TypeSafe | typesafe/jev-1.14-20261014`; context 262,144; `supported_parameters: []`; no native tools. |
| Jev alias / Jev Router endpoint lookup | Empty endpoint arrays. This is not evidence that their resolved models have no provider. |
| `GET /api/v1/benchmarks` without credentials | HTTP 401, `{"error":{"message":"No cookie auth credentials found","code":401}}`. No retry with credentials. |

The catalog documents `output_modalities=text` as the default, `all` for all
modalities, optional pagination, and `links.next`. A negative name search in the
default response alone would have incorrectly concluded that Jev was unavailable.
The Jev hub still says there were 22 decision models on October 16; the fresh
catalog contains 36 entries, including aliases. Neither is a count of independent
benchmark-covered executors. [S1–S4, S9]

Reproduction (GETs only):

```sh
curl -fLsS 'https://openrouter.ai/api/v1/models?output_modalities=decisions'
curl -fLsS 'https://openrouter.ai/api/v1/models?output_modalities=all'
curl -fLsS 'https://openrouter.ai/api/v1/models/typesafe/jev-1.14/endpoints'
```

SHA-256 of the unmodified captured response bytes:

| Request | SHA-256 |
| --- | --- |
| Default models | `0896d7b4be769e67d8548c781f19ea92a31f43b4f01b7eaa81c83363e072b2a4` |
| Decisions models | `6656f67d94a1bd91962e47d6172f0e4f7d8e6c6622642ade247b15a2a1b14a65` |
| All models | `9881f43e6c98b138f5627399eca1b117269d3dcc6f04422d44b7b4ba7e519d60` |
| Jev 1.14 endpoints | `e95ee387c4bb60e207c04553675cccf44bdacce35d75d3dfbb7de9eefa7f2705` |

These hashes identify this research's observations; they do not promise that a
later live response will be identical or provide a substitute for archived bytes.

### Exact names and version policy

- OpenRouter: `typesafe/jev-1.14`; canonical catalog slug
  `typesafe/jev-1.14-20261014`; alias `~typesafe/jev-latest`.
- Creator's own API: `jev-1.14.0`; `jev-latest` currently resolves to that release.
- OpenRouter's TypeSafe compatibility guide explicitly maps bare `jev-1.13` to
  `typesafe/jev-1.13` and bare `jev-latest` to `~typesafe/jev-latest`.
  Prefer the explicit OpenRouter ID for a new OpenRouter integration; do not
  infer an arbitrary cross-provider ID-rewriting rule. [S5, S7]
- Some SDK/cookbook examples still pin 1.13. The current hub, creator model table,
  and catalog identify 1.14 as latest. Examples are not a live availability feed.
- Preserve both requested ID and returned resolved model/provider. A moving alias
  can change answers and invalidate tuned thresholds. Reproducible operation
  favors an explicitly approved version, but the user has not yet chosen a
  release-tracking policy. No inference with either version was performed. [S5]

## 2. Authentication and connection boundary

OpenRouter uses its **own Bearer API key**. A key authenticates both the Decisions
and System One surfaces and charges the OpenRouter account; no separate TypeSafe
account/key is needed when calling through OpenRouter. The creator's direct API
at `api.typesafe.ai` is a separate service and credential boundary. [S1, S6, S7, S10]

A supported connection flow for local apps is OpenRouter OAuth PKCE:

1. Bind the local callback before authorization; the documentation supports
   `http://localhost:<arbitrary-port>/callback`.
2. Open `https://openrouter.ai/auth` with `callback_url`, S256 challenge, and a
   fresh `state`; validate the returned state on the callback.
3. Exchange the short-lived authorization code and verifier at
   `POST https://openrouter.ai/api/v1/auth/keys` for a **user-controlled API key**.
4. Protect that key as a credential and use it as the Bearer token. This is not
   the ChatGPT rotating access/refresh-token protocol. The page documents
   single-use authorization codes expiring after ten minutes, not a ChatGPT-style
   refresh contract for the returned key. [S11]

Manual API-key entry is also documented; choosing the MVP UX remains design work.
No sign-in or exchange was attempted. Validate account/key/workspace access without
turning a failed validation into hidden inference. Credits, rate limits, workspace
policies, provider privacy rules, region, and model access remain runtime gates.
Jev model pages say EU in-region routing is unavailable; do not infer that a
successful global catalog read satisfies a residency requirement. [S1, S10, S11]

For Harness, the safe direction is a distinct Rust-owned OpenRouter connection;
never reuse the ChatGPT token, expose credentials through ACP, or assume a
ChatGPT subscription funds OpenRouter. Existing Harness credential protections
are relevant prior art, not proof of an OpenRouter adapter. See section 9.

## 3. Model, provider, and execution capabilities

These are separate questions:

1. **Model identity/capacity:** ID, canonical revision, modalities, context and
   model-level parameter metadata.
2. **Serving endpoint:** provider, actual supported parameters, limits, pricing,
   moderation/privacy policy, region and recent availability/performance.
3. **Harness execution:** implemented transport and continuation protocol,
   connection, enabled model/effort, available tools and permission policy.

A public catalog entry does not establish (3). A model-level supported-parameter
list does not prove every endpoint supports it. OpenRouter's structured-output
and provider-routing docs explicitly warn that endpoint support differs and can
change. `provider.require_parameters: true` rejects providers lacking requested
parameters; its default is false, and unsupported parameters may otherwise be
ignored. Tool routing without a hard requirement is not sufficient validation.
Provider `only`/ordering/fallback policies must also obey the application's
connection and execution restrictions. [S9, S12, S13]

**Direct Jev:** typed decisions are its native output, not `response_format`
JSON-schema chat output. The Decisions request schema has `model`, `state`,
`questions`, optional assets/provider/trace/session metadata, and no chat tools or
reasoning-effort fields. Jev's catalog parameters are empty and its output is
`decisions`; it cannot serve as Implementer/Reviewer prose/tool execution. [S1–S7]

A metadata trap: the Jev endpoint response nevertheless reports all four
`supports_tool_choice` flags as true. Those generic booleans conflict with using
this endpoint as a chat/tool executor: `supported_parameters` is empty,
`native_tools` is `{}`, and the documented decision schema has no tool calls.
**Do not promote Jev to executable coding-agent status from those flags.** Treat
this as generic/inconsistent metadata and validate against the native protocol;
there is no live tool-call verification in this report. [S4, S6]

For actual execution models, require the **intersection** of creator capability,
endpoint support, and implemented Harness protocol/tool continuation. For example,
OpenAI's current GPT-6 Luna model reference supports function calling and
structured outputs, but limits Chat Completions function calling to
`reasoning_effort=none`; its Responses API supports function calling more broadly.
This illustrates why capability cannot be represented by one model-wide tool
boolean. It does not expand Harness's subscription permissions. [S14]

## 4. Thinking settings: what is and is not supported

**Jev itself has no documented selectable thinking level.** It produces no
reasoning traces or explanations. Neither the Decisions request schema nor its
catalog row advertises effort controls. Do not send `low`, invent a `none`
setting, or confuse missing traces with zero computational work. The selected
worker's thinking level is a **decision output**, not a knob on Jev. [S1, S4–S7]

For execution candidates on OpenRouter, the reasoning guide documents gateway
efforts `max`, `xhigh`, `high`, `medium`, `low`, `minimal`, `none`; these are **not
universally supported values**. Use per-model `reasoning` metadata:

- `supported_efforts`: explicit subset; `null` means all gateway efforts are
  accepted, while omitted means effort selection is not exposed.
- `mandatory: true`: reasoning cannot be disabled; do not offer/send `none`.
- `default_effort`, `default_enabled`: defaults, not an explicit user selection.
- `supports_max_tokens: true`: token-budget control is advertised; otherwise do
  not invent support.
- Creator and endpoint restrictions still matter; normalization can map efforts
  rather than preserve an identical computational budget. [S15]

The same guide distinguishes `reasoning.enabled`, effort, budget, context mode,
and pro mode. These are not interchangeable. `reasoning.exclude` only hides
returned reasoning on Chat Completions; it does **not** disable or unbill thinking,
and Responses ignores that exclusion. Reasoning normally consumes the output
budget and is billed even if no visible answer is produced. `reasoning.mode=pro`
can route to another model variant, so it cannot be an implicit extra setting
outside the approved candidate set. [S15]

Creator cross-check: GPT-6 Luna currently advertises `none`, `low`, `medium`
(default), `high`, `xhigh`, `max`; Harness currently exposes a narrower effort set.
Do not mechanically replace Harness's existing settings with the gateway enum or
silently coerce an inherited setting. [S14, R3]

## 5. Structured selection and failure behavior

The following is a **recommended contract derived from the accepted policy**, not
an implemented Harness interface.

1. Build a bounded, versioned snapshot of **enabled executable candidates only**:
   actual connection + execution model ID/revision + exact supported thinking
   setting + tool/protocol constraints. OpenRouter's entire public catalog must
   never become the executable pool by itself. A decider connection is not a
   worker execution adapter.
2. Compute eligibility, context fit, price arithmetic and policy exclusions in
   code. Give Jev concise semantic task information and prevalidated evidence;
   TypeSafe explicitly warns that numeric arithmetic, date comparison, large
   irrelevant state and indirection are weak areas. [S16]
3. Use one Choice whose opaque option keys map to **valid model/thinking pairs**
   (including connection identity internally), rather than independently choosing
   a model and an incompatible effort. Choice supports up to 255 options; keep
   within that bound or define an explicit further design. An explicit inherit
   outcome can map to the exact parent pair. [S17]
4. Validate response type and required answer key, known option membership,
   finite/ranged probabilities, and any required evidence/threshold fields.
   Missing optional confidence/probabilities remain **missing**, not fabricated
   zero. The Decisions schema also permits per-question refusal; a malformed,
   missing, refused or unacceptable decision is not authorization to execute.
   Revalidate the mapped pair against current enablement/connection/capability
   before dispatch; reject stale/disabled/unexecutable selections. [S6]
5. Record requested and resolved Decider model/provider, snapshot/evidence
   versions, decision timing/usage, chosen pair, and validation/fallback outcome.
   Any readable explanation should describe recorded evidence/policy, not pretend
   to be Jev's reasoning trace.

A schema-conforming result can still be semantically wrong. TypeSafe's own
jaggedness page warns about prompt injection in state and option-order bias.
Keep untrusted ticket/repository text as data, compare reordered/hostile cases in
later probes, and never let it supply new executable identities or permissions.
Confidence is a distribution statistic: for Choice,
`(p_max - 1/n) / (1 - 1/n)`. It is **not** simply the probability that the chosen
answer is correct. Thresholds need workload/version validation; no production
threshold was established here. [S16–S18]

### Parent inheritance is precise, not a general retry policy

- **Unrouted child:** inherit its parent's model **and thinking level**.
- **Failed routing decision:** visibly record why routing failed and inherit
  that same exact pair. This includes a failed Decider request or invalid/stale
  selection, rather than executing an unvalidated choice. If the inherited pair
  itself cannot execute, expose that failure; do not silently lower effort or
  choose another provider/model.
- **Worker/provider/tool execution failure after a valid selection:** preserve
  the failure/uncertain tool outcome. The map does not authorize switching to the
  parent or any other model just because arbitrary execution failed.

OpenRouter's automatic backup-provider behavior and model fallback lists are
separate mechanisms. Their use/retry limits need explicit policy and visible
provenance, not accidental adoption through SDK defaults. `allow_fallbacks` is
true by default for provider routing. TypeSafe SDKs also retry certain errors by
default; decide timeouts/retry budgets deliberately. Do not replay uncertain tool
actions. [S12, S19; HARNESS-1]

### Why not silently use Jev Router instead?

The router's `models` and `allowed_models` **are not hard allowlists**. If matched
models leave no routable pool, it can retry without the includes; `excluded_models`
are hard exclusions. It may select a different effort from the hint, adapt across
turns, or use an advisor. `cost_tier` is a preference, not a per-request spending
cap or effort. Its metadata exposes selected models, candidates and estimated
costs, but that is not a local decision-only API. These semantics do not establish
the map's enabled-executors-only or exact-inheritance guarantees. [S8]

## 6. Context capacity, usage, and latency

**Direct Jev:** OpenRouter publishes a 262,144-token context window. Creator docs
separate default **64k per request** (state + all questions) and **32k per
question** (state + that question). The creator's opt-in long-context header raises
both to 262k, at the same per-token price but with slower and less accurate
behavior. OpenRouter's Jev hub says requests above approximately **32,000 bytes of
text**, and all image requests, enter long-context mode automatically. These are
different contracts and units; neither 32k default limit can be replaced with the
262,144 catalog number in a latency estimate. The exact text-byte threshold/
normalization and compatibility-route behavior were not exercised. [S1, S4, S5]

OpenRouter reports a large generic `max_completion_tokens` for Jev. This is not
permission to generate prose or a useful autoregressive-throughput budget for a
parallel typed-decision model. A router's maximum context is likewise not the
capacity of every resolved endpoint. [S2–S4, S8]

Capacity is not utilization. `context_length` is a limit; request input-token
usage is reported consumption for that generation. Historical cumulative usage
is not the latest request's context occupancy, and neither is account quota.
Reserve space for system instructions, tool schemas/history, output and reasoning
where applicable. Tokenization/hidden request overhead, caching and provider
semantics matter. Unknown token use/context limits must stay unknown rather than
rendering 0% used or claiming guaranteed fit. [S9, S15, S20; R3]

For Jev's low-latency suitability, evidence is encouraging but scoped:

- TypeSafe's launch reports 70–500 ms end-to-end for its examples, with explicit
  caveats about short input, West Coast laptop measurements and advantageous
  demos. Not a Harness SLO or a global p99 guarantee. [S21]
- OpenRouter's **own measured Banking77 evaluation** of Jev **1.13**, run September
  22, reports 175 ms p50, 270 ms p95 and 353 ms p99, measured through OpenRouter
  from one client with eight concurrent requests. Short utterances and a 77-label
  Choice are not full software-factory routing state. [S22]
- The Jev 1.14 public model page displayed **0.11 s p50**, best provider, one-week
  view, all locations at retrieval. Its endpoint API simultaneously had
  `latency_last_30m: null` and `throughput_last_30m: null`; those missing fields
  are not zero and do not contradict a different aggregate/window. This report
  did not measure either value independently. [S4, S23]

Measure complete decision RTT (not just first-token time), p50/p95/p99, failures,
input size, region and cold/warm behavior in a later authorized probe. Jev's
output is not a token stream where tokens/second predicts decision latency.

## 7. Prices, estimates, and actual reported charges

### Listed prices

Jev 1.14 and 1.13 both list **`prompt: "0.000000042"` USD/input token** and
**`completion: "0"`**. Conversion: **$0.042 per million input tokens**, or **$42
per billion**, with free output tokens. Creator and OpenRouter agree. This does
not mean a request has no charge or uses no output tokens. The billed input
includes state/questions and applicable image tokens, not just the user text.
[S2–S5, S7]

OpenRouter catalog rates are decimal strings with units per token, request,
image or search depending on the field. Provider-routing `max_price.prompt` and
`max_price.completion` instead use **USD per million tokens**. Mixing these units
creates a million-fold error. Cache reads/writes, audio/image rates, request/tool
fees, discounts, and conditional `pricing.overrides` may apply; the schema
supports long-context/time conditions and extended cache-write TTL prices.
Missing fields are not free features. A three-field base-price table cannot model
all this. [S9, S12]

Jev Router's catalog uses `-1` prompt/completion prices; its model page says it
has no fixed price and bills the model that serves the request. Treat the negative
values as **non-price/unknown for estimation**, not negative spend, zero cost,
or Jev's $0.042/M rate. [S3, S8]

### Estimate versus measured/account-reported cost

An input/output forecast multiplied by applicable rates is an **estimate**, with
assumptions about token counts, caches, selected endpoint, pricing time and output
length. It is not the amount charged. Keep the Decider's cost separate from worker
cost and include it in full routing/run totals. Do deterministic arithmetic in
code, not Jev. [S16]

OpenRouter's usage accounting returns native-token counts and account cost in
terminal Responses events or non-streaming responses; Chat Completions uses
`prompt_tokens`/`completion_tokens` rather than `input_tokens`/`output_tokens`.
Reasoning/cache details are optional. The old Chat Completions
`usage.include`/`stream_options.include_usage` settings are documented as
unnecessary/deprecated. Direct Decisions/System One responses carry input/output
tokens and may include `id`, resolved `model`, `provider`, `usage.cost`, `is_byok`
and cost details. **The Decisions schema makes `usage.cost` optional**, despite
the hub's statement that every response includes it; consumers must handle its
absence honestly. [S6, S7, S20]

`usage.cost` means what OpenRouter debits in **credits**; the Jev guide expresses
it in USD. Under BYOK it is **OpenRouter's fee only**, not the provider's full
charge. Upstream cost details are separate, sometimes null or unavailable. The
usage guide says `/generation`'s upstream cost is only available for BYOK and is
0/null otherwise: do not interpret that zero as free provider execution. [S6, S20]

`GET /api/v1/generation?id=...` is a documented authenticated audit path for a
generation's `total_cost`, token reports, resolved provider/model and timing.
Whether all Decisions/System One generation IDs have identical retrieval
coverage is **unverified**, so preserve the original terminal report first. No
actual user charges or generation records were queried. [S24]

Persist generation identities and reported usage once; reconcile later metadata
without double counting. Cancelled/superseded/tool-continuation generations still
count when reported. A disconnected stream can lose its terminal usage report:
that is unknown spend or a known lower bound, not a zero-cost success. Existing
Harness's checkpointed usage ledger is relevant prior art. [R3]

Account-debited credits are also not an invoice total. OpenRouter's current
pricing FAQ distinguishes credit-purchase fees (5.5% Standard, 8% Business),
contract pricing and BYOK allowances/fees. Applicable taxes, purchased-credit
fees and external BYOK bills cannot be reconstructed from model token rates
alone. These commercial terms need refresh before presenting a cash-spend total.
[S25]

## 8. Benchmarks, coverage, and evidence refresh

### What can actually be substantiated

- **Creator release/evals:** TypeSafe's launch links its own workflow eval site,
  `https://evals.typesafe.ai/`, and explains reference probabilities from stronger
  models, fixed workflows and possible author bias. The October 14 release says
  its internal intelligence evaluation is evolving, private, likely biased
  toward its model and includes only open-weight comparators. Its benchmark
  policy explicitly rejects a standard benchmark table and favors dated snapshots.
  These are useful provenance and limitations, not a standardized coding score
  or proof of zero wrong judgments. The launch's “no hallucination” claim concerns
  output type/schema constraints, not semantic correctness. [S21, S26, S27]
- **OpenRouter's Banking77 run:** this is a primary report by the evaluator, not
  merely a retold creator claim. Full test split: 3,080 examples, 77 intents;
  Jev 1.13 accuracy 81.0%, macro-F1 80.5%, zero malformed responses; measured total
  charge $0.34. It discloses prompt design, concurrency, cache treatment,
  bootstrap intervals and limitations. Opus used reasoning off and a strict label
  schema. The report explicitly says confidence is not calibrated correctness
  probability and cascade thresholds were evaluated on the same test set.
  **No Jev 1.14 software-routing quality score follows from that run.** [S22]
- **OpenRouter benchmark interface:** `GET /api/v1/benchmarks` documents
  `source=artificial-analysis|design-arena|openrouter`, task filters
  `coding|intelligence|agentic|search`, and OpenRouter GPQA/tau-bench/search
  benchmark types. It requires a valid key and documents 30 requests/minute/key,
  500/day/account. Sources have different response shapes, source URLs, model
  mappings and optional fields. Search results publish the highest-scoring
  eligible configuration, not an arbitrary or necessarily default effort.
  The unauthenticated request returned 401; actual model coverage, freshness and
  Jev inclusion were not obtained. **API existence is verified; score coverage is
  not.** [S28]
- **Original external methodology:** Artificial Analysis publishes the current
  Intelligence Index v5.0.1 methodology: primarily text/English, with specified
  task weights, evaluation prompts/settings, model-specific output limits and
  separate multilingual/visual evaluations. It is a defensible source for the
  measurements it owns, not a universal fit score for every Harness role. When
  OpenRouter republishes such scores, retain its mapping and follow `source_url`
  to the evaluator. No Jev entry or complete eligible-candidate coverage was
  verified on that site. [S29]

Usage popularity, creator marketing ratios, throughput, benchmark scores and
real task success measure different things. Don't map a missing benchmark to
zero, impute it from a similarly named model/revision, or promote a rank from one
effort to all efforts. A capability claim should come from the creator/endpoint;
a benchmark observation should come from the evaluator and its configuration.

### Suggested evidence record and refresh rules (not yet product policy)

For each fact, retain source URL/owner, retrieved-at timestamp, exact model and
revision/alias resolution, endpoint/provider, measurement time/window, units,
workload/benchmark version, effort and inference configuration, and freshness or
failure status. Keep **observed**, **listed**, **estimated**, **reported charge**,
**missing**, **stale** and **not applicable** distinct.

- Refresh catalog/capability/endpoint eligibility before use when the cached
  snapshot expires, or when a selection cannot be revalidated. Resolve aliases
  again at dispatch and retain the model actually used.
- Refresh prices independently of benchmarks; a new price need not invalidate
  a measured score, but can invalidate an estimate.
- Benchmarks need their own version/measurement date and exact-model mapping.
  Keep coverage visible; absence is not low quality and freshness does not cure
  workload mismatch.
- A fetch failure can preserve a timestamped stale display, not turn stale facts
  into verified eligibility. Hard execution constraints require current validation
  or an explicit accepted stale-data policy. Unknown statistics can remain in the
  semantic decision evidence as unknown; they must not become invented numbers.
- No universal TTL or quality threshold was established by this research.
  Define them during design, record snapshot versions, and invalidate tuned
  thresholds on meaningful model/question/candidate-set changes.

## 9. Existing Harness research and implementation boundary

Read against repository starting commit
`6b4512d6ef7b0aa38783ff939bfd0f76234f41a4`:

- [ChatGPT OAuth research](chatgpt-oauth.md) documents OpenAI's subscription
  sign-in, dynamic registration, rotating refresh tokens and preview constraints.
  Its initial-scope note predates the now-documented inference consumer; do not
  mistake that historical note for today's implementation status. [R1]
- [Model pricing research](model-pricing.md), verified November 6, describes a
  manually maintained Standard/short-context **USD per million tokens** snapshot
  in `server/models.json`, not live pricing, actual spend, or subscription charges.
  It already excludes cache-write/tool fees and notes long-context/mode premiums.
  New OpenRouter per-token strings require conversion and richer constraints,
  not blind reuse of those three fields. The current OpenAI Luna reference agrees
  on $0.10/M input, $0.01/M cache read and $0.50/M output, and separately lists
  cache writes, long-context and mode premiums. [R2, S14]
- [Provider connections](../provider-connections.md) and
  [Conversations](../conversations.md) document Rust-owned credentials, one active
  connection/provider, ChatGPT Responses execution, current effort vocabulary,
  strict selection validation, durable usage uncertainty, and no silent retries.
  These are valuable invariants for a later adapter, not proof that its wire
  protocol is interchangeable with OpenRouter. OpenAI's current subscription
  inference docs still require public Responses inference with `store:false`
  and `stream:true`. [R3, S30]
- Source check: `infrastructure/src/create_provider_connection_factory/mod.rs`
  and `infrastructure/src/create_model_step_factory/mod.rs` select only `openai`
  and reject unsupported providers. Thus **current product execution is OpenAI
  through ChatGPT**. A different Harness development harness's providers are not
  this product's runtime capabilities. [R4]

The new OpenRouter connection and direct decision transport remain new work.
Worker execution candidates remain only what Harness actually supports and the
user has enabled. Neither this report nor a public model catalog enables another
provider or model automatically.

## 10. Remaining decisions and verification gates

**Identity is resolved; no replacement model is needed.** Human/design decisions
still required:

1. Confirm the desired Jev release policy: an explicitly approved 1.14 revision
   versus the moving latest alias, with revalidation on updates. The user's name
   alone did not specify version pinning.
2. Choose OpenRouter connection UX (local PKCE or key entry), permitted
   region/data handling and spend/retry limits. Do not request secrets in tracker
   comments or this research thread.
3. Define decision timeout, acceptable routing latency, confidence/abstention
   behavior and stale-evidence policy. Defaults from demos are not accepted policy.
4. Decide whether any worker execution beyond today's ChatGPT/OpenAI is in scope;
   do not let the decider connection answer that question implicitly.

Later **explicitly authorized** verification, outside this no-paid-calls report:
key/workspace access; both decision routes' exact response/error contracts;
resolved revision/provider; cost reporting and generation-audit coverage; short
versus long-context limits and RTT; disabled/stale candidate rejection; omitted
usage/probabilities/refusals; hostile state/option-order changes; pair-preserving
visible routing fallback; and separate worker/provider failure handling. Use
repository-standard temporary external probes, not permanent test additions.

No authenticated usage/cost totals, runtime latency guarantees, complete benchmark
coverage or local Jev selection quality have been established. Those unknowns do
not negate the public identity/availability finding, and must not be filled with
zeroes or statistics copied from a different model.

## Sources

All external sources below were retrieved on 2026-11-06 unless a historical
measurement/release date is stated above. API counts/prices are observations,
not future availability promises.

- **S1** — OpenRouter [Jev documentation hub](https://openrouter.ai/docs/guides/community/jev.md).
- **S2** — OpenRouter [Decisions catalog, live JSON](https://openrouter.ai/api/v1/models?output_modalities=decisions).
- **S3** — OpenRouter [all-modality catalog, live JSON](https://openrouter.ai/api/v1/models?output_modalities=all); [default text catalog](https://openrouter.ai/api/v1/models).
- **S4** — OpenRouter [Jev 1.14 endpoint JSON](https://openrouter.ai/api/v1/models/typesafe/jev-1.14/endpoints); [Jev 1.13](https://openrouter.ai/api/v1/models/typesafe/jev-1.13/endpoints); [latest alias](https://openrouter.ai/api/v1/models/~typesafe/jev-latest/endpoints); [router](https://openrouter.ai/api/v1/models/typesafe/jev-router/endpoints).
- **S5** — TypeSafe [Models, prices, context limits and aliases](https://docs.typesafe.ai/models.md).
- **S6** — OpenRouter [Decisions API schema](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request.md); [System One API schema](https://openrouter.ai/docs/api/api-reference/systemone/submit-a-system-one-request.md).
- **S7** — OpenRouter [TypeSafe SDK compatibility guide](https://openrouter.ai/docs/guides/community/typesafe-sdk.md).
- **S8** — OpenRouter [Jev Router contract](https://openrouter.ai/docs/guides/routing/routers/jev-router.md); [router model/pricing page](https://openrouter.ai/typesafe/jev-router).
- **S9** — OpenRouter [Models API schema, filters, pagination, price/reasoning fields](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties.md).
- **S10** — OpenRouter [authentication](https://openrouter.ai/docs/api_reference/authentication.md).
- **S11** — OpenRouter [OAuth PKCE and local callbacks](https://openrouter.ai/docs/guides/overview/auth/oauth.md).
- **S12** — OpenRouter [provider routing, requirements, prices and fallback controls](https://openrouter.ai/docs/guides/routing/provider-selection.md).
- **S13** — OpenRouter [structured outputs and per-endpoint guarantees](https://openrouter.ai/docs/guides/features/structured-outputs.md).
- **S14** — OpenAI [GPT-6 Luna model reference](https://developers.openai.com/api/docs/models/gpt-6-luna).
- **S15** — OpenRouter [reasoning settings and per-model metadata](https://openrouter.ai/docs/guides/best-practices/reasoning-tokens.md).
- **S16** — TypeSafe [model jaggedness](https://docs.typesafe.ai/model-jaggedness.md).
- **S17** — TypeSafe [Choice primitive and cardinality](https://docs.typesafe.ai/primitives/choice.md).
- **S18** — TypeSafe [confidence semantics and formulas](https://docs.typesafe.ai/confidence.md); [System One concepts/calibration limits](https://docs.typesafe.ai/concepts/system-one.md).
- **S19** — TypeSafe [request rules, errors and retry behavior](https://docs.typesafe.ai/api/errors.md).
- **S20** — OpenRouter [usage accounting and BYOK distinctions](https://openrouter.ai/docs/cookbook/administration/usage-accounting.md).
- **S21** — TypeSafe [Introducing System One Models & Jev, September 15](https://typesafe.ai/blog/introducing-system-one-models-and-jev).
- **S22** — OpenRouter [primary Banking77 evaluation, including methods/caveats](https://openrouter.ai/blog/insights/jev-vs-claude-opus-5-classification/).
- **S23** — OpenRouter [Jev 1.14 pricing/performance page](https://openrouter.ai/typesafe/jev-1.14); [multimodal Decisions constraints](https://openrouter.ai/docs/guides/community/multimodal-decisions.md).
- **S24** — OpenRouter [generation audit API schema](https://openrouter.ai/docs/api/api-reference/generations/get-request-&-usage-metadata-for-a-generation.md).
- **S25** — OpenRouter [pricing and billing FAQ](https://openrouter.ai/pricing).
- **S26** — TypeSafe [Jev 1.14 release and internal-eval caveats, October 14](https://typesafe.ai/blog/jev-1-14); [linked workflow eval site](https://evals.typesafe.ai/).
- **S27** — TypeSafe [benchmark policy/caveats](https://typesafe.ai/blog/antibenchmaxxing).
- **S28** — OpenRouter [unified benchmark API contract](https://openrouter.ai/docs/api/api-reference/benchmarks/list-benchmarks.md); [live endpoint, authentication required](https://openrouter.ai/api/v1/benchmarks).
- **S29** — Artificial Analysis [original intelligence benchmarking methodology](https://artificialanalysis.ai/methodology/intelligence-benchmarking).
- **S30** — OpenAI [ChatGPT subscription models and inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference).
- **R1** — Repository [`docs/research/chatgpt-oauth.md`](chatgpt-oauth.md).
- **R2** — Repository [`docs/research/model-pricing.md`](model-pricing.md).
- **R3** — Repository [`docs/provider-connections.md`](../provider-connections.md), [`docs/conversations.md`](../conversations.md), and [`GLOSSARY.md`](../../GLOSSARY.md).
- **R4** — Repository [`create_provider_connection_factory`](../../infrastructure/src/create_provider_connection_factory/mod.rs) and [`create_model_step_factory`](../../infrastructure/src/create_model_step_factory/mod.rs), inspected at the starting commit above.
