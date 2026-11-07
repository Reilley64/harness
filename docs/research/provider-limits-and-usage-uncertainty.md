# Provider limits and usage uncertainty

Verified **2026-11-07 UTC**, using public first-party docs/catalog GETs only; no authenticated/account, inference, paid, or experimental calls.
Repository baseline: `6b4512d6ef7b0aa38783ff939bfd0f76234f41a4`.
Question: [HARNESS-14](https://app.plane.so/reilley/browse/HARNESS-14/), for [HARNESS-1](https://app.plane.so/reilley/browse/HARNESS-1/) and [HARNESS-7](https://app.plane.so/reilley/browse/HARNESS-7/), including HARNESS-7's provider-only concurrency checkpoint.
Evidence and alternatives, **not product decisions**. The user rejects arbitrary global, Implementer, and Decider concurrency caps.

## Bottom line and vocabulary

**No numeric simultaneous-request entitlement was established for actual ChatGPT subscription inference or native Jev through OpenRouter.** Absence of a
published/introspectable number means unknown, not unlimited or one request.
RPM/TPM measure requests/tokens over time; cumulative credits/budgets measure spend;
context limits measure one input. None can be converted into an advertised number
of concurrent agent slots. OpenRouter additionally documents a cost-weighted
in-flight admission budget, not a fixed concurrent-request count. [O1–O5, R1–R6]

| Path | Documented observable signals | Not established |
| --- | --- | --- |
| ChatGPT subscription → public Responses | Account-specific model list; exact subscription error codes; ChatGPT Usage settings | Remaining plan/app quota API, reset ETA, RPM/TPM headers or simultaneous-request allowance for this flow |
| OpenRouter → Jev native Decisions/System One | Key/budget introspection contract; native usage/error schemas; generic platform limit guidance | User entitlement, Jev numeric concurrency/RPM/TPM, all generic admission details on both native routes |
| Possible OpenRouter workers | Key/budget signals; upstream throttling; protocol-specific usage/errors | Fixed global paid-model capacity or uniform upstream limits |
| Possible direct OpenAI API-key workers | Organization/project/model rate limits, response headers, admin project-limit API, API spend controls | Applicability of those billing/limit contracts to ChatGPT subscription tokens |

## 1. Actual OpenAI subscription path: use its owning contract

- OpenAI documents OAuth **`POST https://api.openai.com/v1/responses`**, `store:false`, `stream:true`, not ChatGPT `backend-api`. Its account-specific
  `GET /v1/models` uses the same OAuth token and returns `models` with visibility,
  slug and display name. A static/public catalog is not live entitlement. [O1]
- A dynamic client registration binds user **and workspace**. Multiple hosts can
  share that client's plan settings/limits; host IDs identify installations, not
  independent quota. Plus users share a **five-hour usage total across apps**,
  including private/OSS apps; no app receives its own allowance. The documented
  five-hour limit does not apply to Pro: this is not a promise of unlimited Pro
  concurrency or exemption from app/policy/other limits. [O2, O3]
- **ChatGPT Settings → Usage** manages plan/credits and app-specific limits. No public programmatic remaining-allowance,
  app-limit inspection, or concurrent-capacity endpoint is documented in the
  examined SIWC contract. No account settings were inspected. [O2–O4]
- Before streaming, admission may return **`{"detail":"..."}`**, not a standard
  API error envelope: 401 permission/identity, 403 policy/region, 503 direct route
  unavailable. Preserve sanitized shape/status/request ID; prose is diagnostic,
  not a stable discriminator. Structured Responses errors use `error.code` and
  `error.param`. Relevant documented cases: [O4]

| Subscription error code | Meaning and documented recovery |
| --- | --- |
| `subscription_sharing_usage_limit_exceeded` (429) | Pause new plan requests; link Usage settings. Could be app-specific; neither total plan exhaustion nor reset time follows from the code. |
| `subscription_sharing_usage_unavailable` (503) | Availability check failed; preserve credentials, bounded backoff. |
| `subscription_sharing_user_unavailable` (503) | Temporary user/workspace lookup failure; bounded backoff. |
| `subscription_sharing_user_not_eligible` (403) | User/workspace/policy ineligible; do not loop requests or OAuth. |
| `subscription_sharing_unsupported_capability` (400) | Inspect `param`; correct unsupported request rather than retrying it unchanged. |

A usage error can arrive as **`response.failed` after streaming begins**. HTTP 200
or initial text is not completed inference; consume the terminal event and retain
partial/unknown usage. No silent conversion to API-key billing occurs. The SIWC
pages do not specify a subscription `Retry-After`/remaining-quota header schema;
standard API-key header behavior below is **not verified for these tokens**. [O1, O4]

The preview forbids `max_output_tokens` (also `background`, `max_tool_calls`, etc.). An API-key recipe reserving a hard output-token budget cannot simply be
ported here. API token prices are not a formula for subscription allowance or
incremental dollars charged to the subscriber. [O5; repository pricing research]

## 2. Planned OpenRouter Jev: native, not chat-shaped inference

Public recheck: **`typesafe/jev-1.14`**, canonical `typesafe/jev-1.14-20261014`; `~typesafe/jev-latest` targets it. One TypeSafe endpoint,
`output_modalities:["decisions"]`, `supported_parameters:[]`,
`per_request_limits:null`. Neither null limits nor generic endpoint tool-choice
booleans establish unlimited concurrency or chat/tool execution. [R1, R2]

Jev uses **`POST /api/alpha/decisions`** or compatibility **`POST /api/v1/systemone`**. Both use an OpenRouter key and bill OpenRouter, not ChatGPT or a required separate
TypeSafe account. Native schemas require `model`, `state`, `questions`; return
`answers`, `model`, `usage`, with optional ID/provider/cost. Both document 402,
403, 429 and server errors through `{error:{code:number,message,metadata?}}`.
Do not import Chat Completions SSE frames or output-token knobs into Jev. [R3–R5]

Their schemas establish error/usage envelopes, **not native-specific numeric rate/concurrency limits or guaranteed retry headers**. General OpenRouter limits
below are platform guidance; exact enforcement, upstream hints and audit coverage
on both native routes remain unverified without authorized runtime evidence.
TypeSafe's own direct API documents 429 and 529 with exponential backoff, but its
separate credential/service contract does not establish OpenRouter entitlement.
Its Python SDK currently defaults to two retries, retry headers enabled and a
30-second total retry budget; SDK defaults are not Harness policy. [R3–R5, T1, T2]

## 3. OpenRouter platform signals, scope, and monetary admission

- **`GET /api/v1/key`** with the inference key documents `limit`, `limit_remaining`
  (nullable per-key spend cap), `limit_reset`, usage totals by UTC period, BYOK
  totals/inclusion, workspace identity, `budgets[]`, `effective_budget`, and
  `free_model_daily_requests`. Budget rows have scope, limit/spend/remaining USD,
  reset interval/instant and BYOK inclusion. **`rate_limit` is deprecated**; do not
  interpret example numbers or sentinel values as usable capacity. These reads
  themselves have per-IP burst limits; polling is not free of throttling. [R6, R7]
- Key spend is not account balance. **`GET /api/v1/credits`** reports account `total_credits`/`total_usage`, but its current reference requires a **management
  key**. Do not require administrative credentials or claim a normal inference
  key can necessarily query balance. No authenticated reads were performed. [R8]
- Extra accounts/keys do not increase globally governed capacity; model limits differ. Free `:free` variants have 20 RPM and 50/day below
  10 purchased credits, 1,000/day at/above that tier (docs also mention a rounding
  tolerance). These are not Jev limits or simultaneous slots. Paid variants have
  no platform request cap per the generic guide, but still face upstream capacity,
  credit/budget admission and DDoS protection: not unlimited service. [R6]
- Successful inference **lacks `X-RateLimit-*` headers**, per that guide.
  Selected free/key-management limit errors include `X-RateLimit-Limit`,
  `-Remaining`, `-Reset`; key-management reset is epoch milliseconds. Per-IP
  bursts lack these headers. Upstream 429 may include `metadata.provider_code`;
  `Retry-After` is supplied when all attempted providers supplied a retry hint.
  Honor it when present; absence does not mean immediate capacity. [R6]
- OpenRouter estimates token-priced requests' cost up front and holds it against
  an **account in-flight spending budget**, including recently completed costs
  during settlement. The amount is a balance fraction with a ceiling; numerical
  fraction/threshold/ceiling are not published here. Rollout is selective, chiefly
  qualifying prepaid accounts, excluding documented free/BYOK and other cases.
  This can reject work even with positive balance and no live requests. [R6]
- Transient **402**: `metadata.limit_source:"openrouter_in_flight_budget"`,
  `reason:"in_flight_budget_exhausted"`; wait for `Retry-After`. By contrast
  `reason:"weight_exceeds_budget"`, source `openrouter_credits`, means one request
  cannot fit; unchanged retries do not help. `openrouter_key_limit` identifies
  exhausted key credits; `openrouter_credits` can also mean balance shortage.
  Branch on source/reason, not human `remedy_hint`. Applicability/weighting of
  native Jev's input-only pricing was not exercised. [R6, R9]
- **403 budget blocks** identify `openrouter_guardrail_budget`,
  `openrouter_workspace_budget`, or `openrouter_key_limit`; metadata includes
  scope, limit/spend and `resets_at` (nullable). No Retry-After is promised for
  hours/days-long budgets. Member budgets aggregate keys; workspace budgets pool
  members/keys; a shared guardrail definition is not one shared allowance.
  Different Decider/worker keys may separate attribution but not shared account,
  member, workspace or upstream capacity. Inspect actual scope, not role. [R6, R10]

Provider budgets are real rejection controls, but **not a strict no-overshoot dollar guarantee**: workspace checks precede
routing, already-dispatched work completes and may overrun the cap. BYOK counting
is configurable and can use OpenRouter-equivalent spend, not the upstream invoice.
Generic spend-controls prose and the detailed limits/key schema disagree on budget
check order; consume returned `effective_budget`/error scope, not an invented order.
No exact cross-route atomic spend reservation guarantee was established. [R6, R7, R10]

## 4. Possible workers and usage/cost uncertainty

Direct **OpenAI API-key** workers need a separate billing/credential adapter. Their docs describe organization/project and model/shared-family RPM/TPM limits,
`x-ratelimit-{limit,remaining,reset}-{requests,tokens}`, optional project-token
headers and temporary-error `Retry-After`. An admin API lists project/model limits;
these are not subscriptions or concurrent slots. API spend alerts do not stop
traffic; current hard org/project limits do return 429 codes
`organization_spend_limit_exceeded` / `project_spend_limit_exceeded`, but propagation
can overshoot. Do not repeat older blanket claims that API budgets are soft. [O6–O8]

OpenRouter worker usage follows its chosen protocol: Responses terminal events
carry input/output tokens and cost; Chat Completions has its own accounting frames.
Native Decisions/System One require token counts in `usage`, but `cost` is optional in their schemas despite Jev hub prose promising it on every response. Record
absence honestly. `usage.cost` is OpenRouter credits (USD-denominated); on BYOK it
is the gateway fee, not total upstream spend. Reported zero output price is not
zero input cost: current Jev catalog input price is $0.042/M tokens. [R1, R3–R5, R11]

Disconnected/cancelled streams may lose terminal accounting. OpenRouter only promises cancellation stops billing on supported streaming providers; other
requests can complete and bill after disconnect. Do not assume native Jev has a
chat-stream cancellation guarantee. `GET /api/v1/generation?id=...` can audit
usage/cost asynchronously, with nullable native counts and possible 404; the read
sources provide no reconciliation-latency SLA or verified Jev coverage. Missing ID,
missing cost or failed audit is unknown, not zero or proof of no charge. Account/key
usage deltas can include other clients and cannot identify a run's bill. [R11–R13]

## 5. Harness constraints and bounded implementation-policy alternatives

Both factories support only `openai`; OpenRouter and API-key workers are not executable at baseline. The OpenAI adapter has no automatic retry, discards non-success
bodies/limit headers, and only preserves sanitized SSE error codes. Its usage reader keeps input/output counts, not cost/cache/reasoning breakdowns.
The durable ledger checkpoints unknown before dispatch and retains reported failed/cancelled work. Credential locking serializes rotating refresh, not inference;
per-conversation reply exclusion protects conversation integrity, not provider-wide capacity. Preserve these distinctions. [H1–H3]

Alternatives for subsequent design, without choosing product policy:

1. **Reactive provider admission:** dispatch eligible work without fixed role/global
   slots; persist actual scope/status/code/request ID and cooldown. Queue only
   after provider rejection or a documented exhausted quota. Unknown capacity is
   displayed as unknown; no synthetic “remaining concurrent slots.”
2. **Observable quota scheduling:** use authorized key/budget/model snapshots and
   observed retry/reset signals to avoid known rejection. Snapshots are not leases:
   other processes/apps can consume shared quota. Never infer worker access from
   catalogs, multiply allowance per child/key, or assign one Decider slot.
3. **Recovery classification:** subscription usage-limit 429 pauses plan requests;
   it is not a generic retry timer. Temporary 429/503 use valid Retry-After or bounded exponential backoff/jitter;
   retry in-flight-budget 402 only with the documented source/hint, not generic 402.
   Attempt/time bounds need policy, not arbitrary concurrency defaults. Avoid
   nested SDK retries; preserve cancellation/deadlines. Do not replay partial model
   output or uncertain tool mutations; preserve each potentially billed attempt.
4. **Spend enforcement choices:** rely on explicit provider controls with disclosed
   overshoot/coverage, or add clearly labelled best-effort estimated accounting.
   A fail-closed gate for missing spend data versus proceeding with visible unknown
   exposure requires human policy. Neither is a fabricated provider concurrency
   limit. Strict aggregate dollars cannot be promised from missing/delayed reports,
   unsupported output bounds or unbounded external consumers. Count routing,
   reviews, children, retries and cancelled work in the agreed accounting scope.

Later authorized verification needs entitlement and both native routes' limits/headers/budget enforcement, retry hints, cancellation and audit coverage.
This research does not authorize calls, set dollar/retry thresholds, enable workers, or broaden accepted visible routing fallback into failure failover.

## Sources (retrieved 2026-11-07; floating docs unless version stated)

- **O1** — [SIWC models/inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference).
- **O2** — [SIWC client/host scope](https://developers.openai.com/siwc/token-sharing-open-source).
- **O3** — [SIWC accounts, usage and shared five-hour limit](https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions).
- **O4** — [SIWC errors/recovery](https://developers.openai.com/siwc/token-sharing-open-source/errors-and-recovery).
- **O5** — [SIWC preview restrictions](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations).
- **O6** — [OpenAI API rate limits](https://developers.openai.com/api/docs/guides/rate-limits).
- **O7** — [Admin project/model limit listing](https://developers.openai.com/api/docs/api-reference/project-rate-limits/list).
- **O8** — [OpenAI API spend limits and propagation caveat](https://developers.openai.com/api/docs/guides/spend-limits).
- **R1** — [Live decisions catalog](https://openrouter.ai/api/v1/models?output_modalities=decisions).
- **R2** — [Live Jev endpoint metadata](https://openrouter.ai/api/v1/models/typesafe/jev-1.14/endpoints).
- **R3** — [Native Decisions schema](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request.md).
- **R4** — [Native System One schema](https://openrouter.ai/docs/api/api-reference/systemone/submit-a-system-one-request.md).
- **R5** — [Jev hub, auth/billing and endpoint distinctions](https://openrouter.ai/docs/guides/community/jev.md).
- **R6** — [OpenRouter limits, headers, in-flight admission and errors](https://openrouter.ai/docs/api/reference/limits.md).
- **R7** — [Current key schema](https://openrouter.ai/docs/api/api-reference/api-keys/get-current-api-key.md).
- **R8** — [Credits endpoint and management-key requirement](https://openrouter.ai/docs/api/api-reference/credits/get-remaining-credits.md).
- **R9** — [Errors/retry and protocol-specific envelopes](https://openrouter.ai/docs/api/reference/errors-and-debugging.md).
- **R10** — [Workspace budgets/overshoot](https://openrouter.ai/docs/guides/features/workspaces/workspace-budgets.md), [guardrail scope](https://openrouter.ai/docs/guides/features/guardrails.md), [spend-controls prose](https://openrouter.ai/docs/guides/best-practices/spend-controls.md).
- **R11** — [Usage accounting and BYOK](https://openrouter.ai/docs/cookbook/administration/usage-accounting.md).
- **R12** — [Generation audit schema](https://openrouter.ai/docs/api/api-reference/generations/get-request-&-usage-metadata-for-a-generation.md).
- **R13** — [Streaming and cancellation/billing limits](https://openrouter.ai/docs/api/reference/streaming.md).
- **T1/T2** — TypeSafe [direct API errors](https://docs.typesafe.ai/api/errors.md), [Python SDK retry contract](https://docs.typesafe.ai/sdk/python/api/retries.md); not an OpenRouter server guarantee.
- **H1** — Repository [connection factory](../../infrastructure/src/create_provider_connection_factory/mod.rs), [model-step factory](../../infrastructure/src/create_model_step_factory/mod.rs), [OpenAI adapter](../../infrastructure/src/create_model_step/openai.rs), [usage reader](../../infrastructure/src/create_model_step/usage.rs), inspected at baseline above.
- **H2** — [Provider connections](../provider-connections.md), [conversations/ledger](../conversations.md), [OAuth research](chatgpt-oauth.md), [pricing snapshot](model-pricing.md).
- **H3** — Prior [Jev/provider research, commit 20cf198](https://github.com/Reilley64/harness/blob/20cf19800d0a02257fe523e6c9088d23693c7e19/docs/research/openrouter-decider-routing.md), read without merging; its authenticated access/cost unknowns remain open.
