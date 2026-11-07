# OpenRouter Jev “balanced” routing: mechanisms and boundaries

Research for [HARNESS-15](https://app.plane.so/reilley/browse/HARNESS-15/), supporting [HARNESS-8](https://app.plane.so/reilley/browse/HARNESS-8/).
Retrieved **2026-11-07, approximately 01:06–01:09 UTC**. Evidence, not an adoption or product-policy decision.
Read both tickets and HARNESS-8's checkpoint, repository instructions, and the prior reports at
[`20cf198`](https://github.com/Reilley64/harness/blob/20cf19800d0a02257fe523e6c9088d23693c7e19/docs/research/openrouter-decider-routing.md) and
[`e6f329a`](https://github.com/Reilley64/harness/blob/e6f329a28ba2c913490ce037af8a851f0bae4c98/docs/research/provider-limits-and-usage-uncertainty.md), without merging.
Public official documentation, HTML, API and GitHub source GETs only; no credentials, account inspection, inference, installations, browser automation or runtime experiments.

## 1. What “balanced router” currently names

**Documented fact:** the hosted model is **`typesafe/jev-router`**, with optional plugin **`id: "jev-router"`**.
Its named cost policies are **`low`, `medium`, `high`**. The exact medium description is **“Balance estimated cost and capability.”**
The low description is “Prioritize lower estimated cost”; high favors capability when expected benefit justifies added cost. [J]
**Interpretation:** `cost_tier: "medium"` is the documented balanced policy, not an API mode named `balanced`.
No `balanced` tier/preset was found in the Jev contract or its public documentation source; an unrecognized tier returns `400`.
Omission uses a **default policy**, which the docs do **not equate to medium**. Cost tier is not reasoning effort, a quality guarantee, or a spending cap. [J]

These are different interfaces, not interchangeable model names:

| Interface | What it does | Selection controls |
| --- | --- | --- |
| Native Jev Decisions/System One | Evaluates caller-supplied state and typed questions; returns answers/probabilities, not worker execution | Caller selects decision model and defines questions/options; no built-in balanced policy or Jev thinking-level parameter documented |
| Hosted `typesafe/jev-router` | Selects **and invokes** a generation model and effort; returns that model's response | Cost tier, model include preferences/hard exclusions, provider restrictions; may adapt or use an advisor |

Native routes are `POST /api/alpha/decisions` and TypeSafe-SDK-compatible `POST /api/v1/systemone`.
They use the OpenRouter key/account, not a separate TypeSafe account. Hosted Jev supports Responses, Chat Completions and Messages, streaming or non-streaming. [J, H, D, S]
The Jev Router page advertises “balancing quality, speed, and cost”; the policy documentation establishes qualitative tradeoffs, **not an exposed optimization formula**. [J, P]

### Identity and user-selected release

Fresh Decisions catalog: `typesafe/jev-1.14`, canonical **`typesafe/jev-1.14-20261014`**;
`~typesafe/jev-latest` resolves to it; `typesafe/jev-1.13` remains listed. TypeSafe calls the release **`jev-1.14.0`**. [C, T]
Jev's endpoint advertises `supported_parameters: []`, output `decisions`, and one TypeSafe endpoint.
Native Jev has no documented effort knob or reasoning traces. A worker's selected effort is an **answer/application setting**, not Jev's effort. [E, H, D]
The hosted page links its engine to `~typesafe/jev-latest`; its plugin offers **no documented selector for the underlying Decider release**. [P, J]
**Implication for the accepted user control:** direct native requests expose a `model` choice; choosing the hosted router is not equivalent to choosing/pinning the Decider.
Picker scope, explicit revision versus moving alias, and update behavior remain user decisions. TypeSafe warns alias answers can change and advises pinning when thresholds are tuned. [T]

## 2. Hosted policy: what is actually established

Jev evaluates the conversation for task needs, difficulty and potential benefit of a stronger model.
Policy combines those judgments with estimated cost, available capabilities and preferences. Available models and policy can change. [J]
No hosted internal state/question schema, full decision prompt, cost/capability weights, difficulty cutoffs, success-probability model,
price forecast formula, numerical pool cap, benchmark-to-tier mapping or release-pinning contract was found in the inspected official materials.
The public GitHub source [JS] is **documentation**, not the hosted routing implementation; it cannot establish those internals.

| Concern | Exact documented behavior and consequence |
| --- | --- |
| Includes | `models` and alias `allowed_models` are combined **“include preferences with fallback, not hard allowlists.”** Empty/omitted means unrestricted includes. |
| Relaxation | If includes match catalog models but leave no routable pool, rebuild without includes; metadata: `list_fallback: "models_ignored"`. This includes a matched model with no eligible endpoint. |
| Invalid includes | If no entry matches any public text catalog model, return `404`; do not relax a misspelling/image-only entry into the entire pool. |
| Exclusions | `excluded_models` are hard, retained during include fallback. If they remove everything, fail `404`. |
| Dynamic pools | Filter before ranking and pool cap; includes do not bypass eligibility, endpoint policy, capabilities or token-budget checks. Explicit operator/built-in pools keep configured membership. |
| Reduced adequacy | Lists can lower the capability requirement; metadata `list_tier_cap` is **not** the cost tier. A constrained pool does not guarantee unchanged task adequacy. |
| Identity matching | Base slug matches its dated revisions; dated slug matches that revision only; `~author/family-latest` matches family revisions; wildcards are case-sensitive. |
| Effort | Top-level `reasoning.effort` (legacy `reasoning_effort` on Chat Completions) is a **hint**; final effort can differ and change across turns. `reasoning.max_tokens` and `reasoning.mode: "pro"` return `400`. |
| Fallback chain | Metadata `resolved_models` is in fallback order. This is not an exact-parent-pair routing-failure contract. |
| Advisor | Supported agentic policy can add `advisor_model`; `max_fallback: "deep"` means a would-be advisor request ran without one. Trigger, model set, limits and complete charge attribution are not specified here. |
| Availability | Global data region only; workspace/router and selected-model access still required. Docs list `400`, `404`, `503`; catalog presence is not verified runtime access. |

All rows above are from the Jev-specific contract [J], not inferred from generic field names.
Author aliases such as `anthropic/auto`, **when catalog-listed/access-authorized**, impose hard author scope, including include relaxation;
that is not an arbitrary enabled-model/effort allowlist, and an author is not the serving provider. Separate requested fallbacks keep their own semantics. [J]
Generic provider routing independently defaults `allow_fallbacks:true`, `require_parameters:false`; `provider.only` restricts providers, not enabled local model/effort pairs.
Unsupported parameters can otherwise be ignored. Provider `max_price` is an endpoint unit-price filter, not a whole-task dollar cap. [V]
**Interpretation:** a complement exclusion list would require a complete, revision-aware changing catalog and still would not enforce effort, local connection or exact-parent fallback.
It is not evidence of a safe equivalent to a local positive pair allowlist.

### Temporal selection, revisits and stability

Jev explicitly can retain a model, change effort or switch models as an agentic task progresses; changing cost tier can change model within a conversation. [J]
It does not publish an exact revisit cadence, minimum dwell time, every-N-tool-call rule, rerouting timeout or worst-case latency/SLO.
Native latency also depends on state size: OpenRouter says about 32,000 text bytes or any image activates slower, less accurate long-context mode;
TypeSafe's direct default limits are 64k/request and 32k/question, distinct from the 262,144-token OpenRouter catalog capacity. No timing was measured here. [H, T]
Native questions are evaluated independently in parallel against the same state. Their `session_id` is observability grouping, **never sent to the provider**;
it is not evidence of an automatically adaptive native routing session. Local code chooses when to ask again and what history to send. [D, ST, T]

Related but **not identical contracts**: the official routing-benchmarks HTML says **“Auto Router is now powered by Jev Router.”** [RB]
Current Auto documentation describes eleven task ratings in one call, live prices, cache status and benchmark performance;
it can reconsider every turn, favors effort changes before cache-expensive model switches, and names task changes/repeated errors/failing tests as switch signals. [A]
Those are concrete public policy patterns, but **do not transplant Auto semantics wholesale**: Auto's `allowed_models` is documented as hard,
whereas Jev's is explicitly soft; Auto's deprecated `pin_model`/`cost_quality_tradeoff` are not a native Jev policy API. [A, J]
No source establishes a universal timer, deterministic identical-request result, or personalized online learning of the hosted policy.
TypeSafe says native Jev has the same weights for every account, not customer fine-tuning/LoRA; request-specific adaptation is not proof of weight updates. [T]

## 3. Official typed schemas and reusable policy patterns

Native envelope requires `model`, `state`, `questions`; state is a string/object/array.
Each named question has `type` and primitive-specific `instructions`/`criteria`; Choice criteria map **option key → description**, up to 255 options.
Instructions/descriptions can themselves be structured JSON. Response has `model`, keyed `answers`, and `usage`.
OpenRouter's schema makes Choice `confidence`/`probabilities`, response `id`/`provider`, and `usage.cost` optional; per-question `refusal` is possible. [D, S, CH]
TypeSafe recommends descriptive named state, explicit narrow questions and code-owned control flow; no reliance on latent knowledge when current facts can be supplied. [ST, B]

**Published examples, not recovered hosted-router prompts:**

- **Companion Noul:** a Choice always chooses even when nothing fits. Ask an independent Noul whether any option fits; code ignores Choice when it does not.
  The worked product-routing example shows a confident Choice for an out-of-scope desk lamp and a Noul of `0.02` rejecting applicability. [N]
- **Independent adequacy judgments:** the companion pattern also supports one Noul per option. This can expose “several adequate” or “none adequate” instead of forcing a winner. [N]
- **Composite scoring:** separate semantic dimensions, normalize and combine using **weights controlled in code**. The published résumé weights are examples,
  not Jev Router's proprietary objective and not a prescribed cost/intelligence ratio for Harness. [CS]
- **Draft → verify → escalate:** OpenRouter's cascade sends state `{help_center_excerpts, customer_question, assistant_answer}`;
  its Choice asks **“Compare assistant_answer against help_center_excerpts. Which one describes it?”**, with `supported`, `unsupported`, `declined` criteria.
  Code validates labels, accepts supported answers above its example threshold, otherwise escalates once or hands off. The example pins Jev **1.13**, not current latest. [CA]
- **Batch atomic questions:** independent questions share a state and run in parallel, reducing serial model round trips.
  Output of one question cannot condition another within that request; dependent stages need code/separate calls. [B, ST, T]

**Possible local adaptations, not a chosen policy or official hosted prompt:** prefilter to enabled executable `(connection, model/revision, thinking)` pairs;
ask adequacy/suitability questions over concise task and evidence state; let code compare known estimated costs among adequate pairs.
Alternatively use one Choice with opaque keys mapped to validated pairs, plus an applicability/abstention question.
Code, not Jev, can perform context fit, currency/unit conversion, freshness checks, exact comparisons and final membership validation. [N, CS, B, G]
This preserves inspectable semantic judgments without pretending a weighted sum is the hosted balanced algorithm.
Worker effort eligibility needs per-model/endpoint/protocol evidence: OpenRouter's `reasoning.supported_efforts` is model-specific;
`mandatory:true` forbids disabling reasoning. Gateway effort names do not promise identical computational budgets across models. [RE]
A lack of adequacy/evidence needs an explicit outcome; neither a confident forced Choice nor an invented benchmark zero establishes suitability.

### Numerical cost, confidence and prompt pitfalls

TypeSafe's exact warning is **“Jev is not a calculator.”** It documents unreliable counts, numeric precision/date comparison,
large irrelevant state, indirection, adversarial state and option-order bias. Keep arithmetic/dates in code; send computed facts or meaningful buckets;
option reordering/hostile state need later authorized evaluation, not faith in schema compliance. [G]
Native Jev currently lists `$0.000000042` per input token = **$0.042/M**, output free; hosted router lists `-1` token prices and bills the serving model,
not that native Jev rate. Negative sentinel rates are not savings or negative charges. Worker/Decider/retry/advisor costs must not be conflated. [C, E, P]
`candidates[].estimated_cost` is explicitly **not final billed cost**. Forecast token counts, cache treatment, uncertainty and internal price refresh timing are not exposed in the Jev contract. [J]
Missing cost is unknown, not zero; native `usage.cost` is OpenRouter credits and on BYOK means gateway fee only. API list price does not measure ChatGPT subscription marginal spend. [D; accepted boundary]

TypeSafe defines Choice confidence as `(p_max - 1/n) / (1 - 1/n)`: concentration relative to a uniform distribution,
**not the probability the selected answer is correct**. Choice confidence and top probability are not independent evidence when option count is fixed. [CF]
There is a documentation conflict: the cascade calls confidence “Jev's probability that the chosen label is correct”; the owning confidence guide gives the formula,
and OpenRouter's measured Banking77 report explicitly says confidence is **not calibrated probability**. Do not copy that cookbook gloss or its `0.8` threshold as a guarantee. [CA, CF, BA]
TypeSafe describes probabilities as trained for calibration **across groups**, not guaranteed individual correctness. Thresholds require workload/model/question/candidate-set validation. [SO, CF]

## 4. Evidence: what supports “balanced,” and what does not

- **Hosted Jev:** qualitative policy, observable candidate estimates/reasons and adaptive behavior are documented. The inspected docs publish no medium-tier quality/cost frontier,
  reproducible software-task evaluation, benchmark freshness SLA, complete candidate score coverage or numerical success/cost objective. [J]
- **Router benchmark page:** official docs promise periodically repeated coding/agentic/multimodal router-versus-baseline runs.
  Our public HTML GET returned the page shell/engine statement but no score rows; no Jev medium result, run configuration or date was verified. [R, RB]
- **OpenRouter benchmark API:** requires a valid key; this unauthenticated recheck returned **401**, with no credential retry.
  Schema supports Artificial Analysis, Design Arena and OpenRouter sources; task filters include coding/intelligence/agentic/search.
  Metadata has dataset `version`, `as_of`, attribution and source URL; OpenRouter runs have `last_run_timestamp`.
  AA items expose nullable indices without per-item run/effort metadata. Search publishes the best eligible configuration, not necessarily a worker's chosen effort.
  These are evidence plumbing, **not proof Jev uses this exact feed or that every enabled pair has a fresh score**. [BM]
- **Banking77:** OpenRouter's September 22 run of **Jev 1.13**, 3,080 short utterances/77 labels, measured 81.0% accuracy,
  175 ms median/270 ms p95/353 ms p99 RTT under eight concurrent requests. Its confidence cascade used the same test split to examine thresholds;
  the report warns that is an upper bound. This is classification evidence, **not Jev 1.14 coding-route accuracy or an optimal medium policy**. [BA]
- **Creator evaluations:** October 14's Jev 1.14 internal intelligence measurement is private, evolving, open-weight-comparator-only and explicitly likely biased. [REL]
  Public WorkflowEvals code measures agreement against model/consensus references on invoice, support, agent-trace and security workflows;
  README defaults to Jev 1.13 and records dataset revision, model/settings and unknown costs. It does not benchmark hosted medium routing over Harness tasks. [WE]
- **Worked cascade:** 50 hand-written excerpt-based questions, model-judged; published noise, handoff and forced-escalation caveats.
  Its cheaper routing result is an example, not a transferable benchmark of coding adequacy or a general guarantee. [CA]

## 5. Fit to accepted Harness boundaries and open facts

**Reusable mechanisms:** hard eligibility before semantic judgment; explicit model/effort pairs; applicability/adequacy separate from preference;
semantic signals plus deterministic cost arithmetic; recorded evidence/selection provenance; deliberate stickiness versus revisits.
None chooses the user's policy weights, thresholds, unknown-evidence behavior, revisit trigger or timeout.

**Hard mismatches if hosted Jev were substituted unchanged:** soft includes versus enabled-only execution; effort hints versus exact approved pairs;
selection **plus execution/advisor/fallbacks** versus a local decision seam; hidden engine release versus a user-selected Decider;
adaptive switching and lowered capability requirements versus explicit task selection and actual-parent inheritance.
It also cannot execute a locally authorized ChatGPT subscription connection merely because an identically named API model exists.
OpenRouter workers through factories and enable/disable controls are accepted, but do not authorize that substitution.

For this research's boundary, an unrouted child inherits its **actual parent's model AND thinking level**; routing failure visibly falls back to that exact pair.
A disabled/unavailable inherited pair is an exposed failure, not permission to switch worker, provider or effort.
Failure after valid worker dispatch is **execution failure**, not routing-failure fallback authority. Hosted include/provider/model fallback semantics do not supply this contract.
No findings justify arbitrary app concurrency slots or whole-spec time/spending caps; provider rejection/admission/retry accounting remains separately owned. [HARNESS-8; L]
Browser PKCE is accepted and documented; it yields a protected user-controlled OpenRouter key, not model entitlement or permission for hidden connection-time inference. [O]

**Still unresolved:** exact hosted prompt/weights, benchmark feed/coverage/refresh and price forecast, revision selection, revisit/advisor triggers,
latency tails, native access/error/cost/audit behavior, model-specific threshold calibration, and local software-factory suitability.
Later runtime verification needs separate authorization. User still chooses Decider scope/release tracking, routing evidence/unknown policy, confidence handling,
selection cadence and balanced adequacy/cost tradeoff; this report neither selects those nor enables execution.
For provenance, hosted metadata opt-in exposes requested/resolved models, candidates/efforts/tiers/estimates, reason, pipeline, advisor and include relaxation;
cache hits strip metadata and some failures omit it. Preserve absence honestly; metadata describes a decision, not Jev's reasoning trace. [J, M]

## Sources and versions

All retrieved 2026-11-07; floating docs/live catalogs unless pinned. Quotes above are source wording; implications/adaptations are explicitly interpretations.

- **J / JS** — [Jev Router contract](https://openrouter.ai/docs/guides/routing/routers/jev-router.md); [owning documentation source at `22c53d8`](https://github.com/OpenRouterTeam/docs/blob/22c53d87287b8b1eaa223f414f7580840885d16c/guides/routing/routers/jev-router.mdx), freshly read.
- **H / P** — [Jev hub](https://openrouter.ai/docs/guides/community/jev.md); [Jev Router model/pricing page](https://openrouter.ai/typesafe/jev-router).
- **D / S** — [Decisions request/response schema](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request.md); [System One schema](https://openrouter.ai/docs/api/api-reference/systemone/submit-a-system-one-request.md).
- **C / E** — [Live Decisions catalog](https://openrouter.ai/api/v1/models?output_modalities=decisions), [text catalog containing router](https://openrouter.ai/api/v1/models?output_modalities=text); [Jev 1.14 endpoint JSON](https://openrouter.ai/api/v1/models/typesafe/jev-1.14/endpoints).
- **T / SO** — TypeSafe [models/releases, aliases, context and customization](https://docs.typesafe.ai/models.md); [System One/calibration scope](https://docs.typesafe.ai/concepts/system-one.md).
- **ST / CH / B** — TypeSafe [state](https://docs.typesafe.ai/concepts/state.md), [Choice](https://docs.typesafe.ai/primitives/choice.md), [code-owned workflows and atomic questions](https://docs.typesafe.ai/concepts/how-to-build-with-system-one.md).
- **N / CS / G / CF** — TypeSafe [companion Noul](https://docs.typesafe.ai/patterns/companion-noul.md), [composite scoring](https://docs.typesafe.ai/patterns/composite-scoring.md), [model jaggedness](https://docs.typesafe.ai/model-jaggedness.md), [confidence formulas/limits](https://docs.typesafe.ai/confidence.md).
- **CA** — OpenRouter [Jev-verified cascade, prompts, code and evaluation caveats](https://openrouter.ai/docs/cookbook/evaluate-and-optimize/jev-verified-cascade.md).
- **RE** — OpenRouter [reasoning controls and per-model supported efforts](https://openrouter.ai/docs/guides/best-practices/reasoning-tokens.md).
- **V / M / L / O** — OpenRouter [provider routing](https://openrouter.ai/docs/guides/routing/provider-selection.md), [router metadata](https://openrouter.ai/docs/guides/routing/routers/router-metadata.md), [provider limits/admission](https://openrouter.ai/docs/api/reference/limits.md), [OAuth PKCE](https://openrouter.ai/docs/guides/overview/auth/oauth.md).
- **A / R / RB** — [Auto's distinct contract](https://openrouter.ai/docs/guides/routing/routers/auto-router.md), [routing overview](https://openrouter.ai/docs/guides/routing/routers/overview.md), [public router-benchmark page](https://openrouter.ai/benchmarks/routers).
- **BM** — [Benchmark API contract](https://openrouter.ai/docs/api/api-reference/benchmarks/list-benchmarks.md); [live endpoint, observed 401](https://openrouter.ai/api/v1/benchmarks).
- **BA / REL** — OpenRouter [Banking77 primary evaluation, September 22, 2026](https://openrouter.ai/blog/insights/jev-vs-claude-opus-5-classification/); TypeSafe [Jev 1.14 release, October 14](https://typesafe.ai/blog/jev-1-14).
- **WE** — TypeSafe [WorkflowEvals README at `0ac3b8a`](https://github.com/typesafe-ai/WorkflowEvals/blob/0ac3b8ad845429f0d8e064ecfb2430a47c5a25cb/README.md), freshly read public source; nothing executed.
