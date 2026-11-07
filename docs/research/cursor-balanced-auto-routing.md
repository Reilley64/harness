# Cursor Auto / Balance routing: primary-source findings

Retrieved **2026-11-07 UTC** for [HARNESS-16](https://app.plane.so/reilley/browse/HARNESS-16/), informing [HARNESS-8](https://app.plane.so/reilley/browse/HARNESS-8/).
Scope: public, unauthenticated official documentation, changelog and blog GETs only. No application inspection, account access, model requests, installations or runtime verification. Dated posts below are the versions currently served at retrieval, not archived original copies; undated docs are mutable snapshots, without a published router build/version.

## Bottom line

- **Balance is a named product mode**, not merely an interpretation of “balanced routing.” Current UI modes are **Cost, Balance, Intelligence**. SDK spelling is `balanced`, not the UI label. The TypeScript reference explicitly says: “Use **Balance** in product copy. Use `balanced` only as the SDK wire value.” [S1, S2]
- The present **Cursor Router** is a July 22, 2026 relaunch of Auto for **Teams and Enterprise**. Individual plans have classic Auto/Cost only, subject to plan eligibility. Do not project Balance back onto historical Auto. [S1, S3, S4, S8]
- Cursor publishes considerably more than a vague quality/cost promise: an August 6 research post describes Compass complexity prediction, task taxonomy, candidate eligibility and a traffic-level cost optimizer. It does **not** disclose enough parameters, training data or code to reproduce production policy. [S5]
- Public evidence is Cursor's own production-evaluation reporting, not independent verification or a portable benchmark proving Harness suitability. Model pool, billing and some descriptions differ across current pages; retain those discrepancies rather than inventing a unified implementation. [S5–S9]

## Naming, dates and historical behavior

| Source/version | Documented behavior at that source | Limit on inference |
| --- | --- | --- |
| July 4, 2025 pricing clarification [S10] | June 16 Pro changes included unlimited Auto and $20 frontier usage; “Auto automatically routes to different frontier models based on capacity.” | Historical capacity-routing statement, not evidence of task-aware Balance or today's billing. |
| August 12, 2025 update [S11] | Auto was priced like premium models December 2023–June 2025; unlimited for individuals since June 2025; to consume monthly usage at token rates at the next renewal **after September 15, 2025**. | Effective date depended on renewal, including annual plans; not a universal September cutover. |
| June 1, 2026 Teams pricing [S12] | Separate first-party pool for Auto/Composer and third-party API pool; existing customers transition on cycles starting July 1. | Historical plan accounting, not the current unified-subscription rule. |
| July 22, 2026 Router launch/changelog [S3, S4] | Introduces Cost/Balance/Intelligence; per-request task/complexity classification. Teams on by default; Enterprise opt-in. Desktop, web, iOS, CLI, SDK. | Changelog has a dated feature release, **no numeric editor build** attached. It lists Grok 4.5 as required then. |
| August 6, 2026 research explanation [S5] | Names two new configurations, Auto Balance and Auto Intelligence; describes improved Compass and adding Opus 5 since launch. | Explanation of the system then, not a promise of unchanged parameters today. |
| November 7, 2026 retrieval [S1, S6, S8, S16] | Legacy Auto is called Cost; new-user default is Balance where Router is available. Required efficient model is now Grok 4.6. | Account/team controls can override defaults; public docs cannot establish an account's actual catalog or access. |

The phrase “balance intelligence, cost, and reliability” also appears as a generic Auto description [S6]. That wording alone would not establish a preset; the explicit mode lists and release announcement do. Cost's description, “Uses the previous Auto routing logic” [S1], does not prove today's Cost implementation is identical to the 2025 capacity router.

## Current documented selection contract

### Objectives, inputs and granularity

- Reference wording: Balance “Optimizes for intelligence, speed, and cost”; Intelligence selects more capable models for hard work at lower cost than a single frontier model; Cost optimizes token spend. Balance/Intelligence consume usage limits faster than Cost. No fixed public quality floor or latency SLO is specified. [S1]
- The classifier runs on **each agent request**, using task type and complexity. Launch inputs include query, context, domain and model behavior; research inputs include current turn, recent conversation state and tool calls. This is not documented as a one-time task/worker assignment. [S1, S3, S5]
- SDK docs explicitly say “The underlying model can change between requests.” Help says it can vary between turns. A user can change optimization mode at any time; SDK mode overrides operate **per run**, and later sends inherit that override. Those are different layers: user policy selection versus underlying request routing. [S1, S2, S6]
- **Unknown:** exact mapping of “request”/“turn” to every internal inference in a tool loop, classifier token/input schema, context truncation, treatment of subagent work, and whether a partially streamed response can change model. The public request-level promise does not establish intra-response switching.
- Effort is not an advertised independent Router mode/control. SDK reasoning parameters are model-specific and discovered from the catalog; Router documents `optimize_for`. **Unknown:** routed-model effort budgets, whether effort is itself predicted, or effort preservation across switches. Intelligence is not evidence of “high thinking.” [S1, S2]

### What Cursor actually discloses about its algorithm

The August 6 research post [S5] states:

1. **Compass** predicts user satisfaction as a complexity proxy, with a score from 0 to 1. A threshold divides efficient-model work from frontier routing. Its wording says lower thresholds keep more traffic on the efficient model and higher thresholds upgrade more often; do not reinterpret the score as a conventional increasing difficulty scale.
2. A taxonomy describes **domains, tasks and modifiers** (e.g. backend, bug fixes, bounded edits). It is learned from real developer traffic, not merely public benchmark rankings.
3. Exact published eligibility wording: a candidate must clear a “**one-sided 75% uplift threshold**” against the efficient model, explained as roughly 75% confidence that improvement is real. This is **not** a claim of 75% performance improvement.
4. An optimizer chooses a **traffic-weighted combination** with greatest expected performance gain while keeping **average cost per turn** within the mode's budget. Balance keeps more work on the efficient path and gives the task router a smaller budget than Intelligence.
5. Training/evaluation cost includes token use and cache misses caused by model switching. Cross-validation and held-out evaluation precede live traffic evaluation.

These are first-party descriptions, not independently observed execution. Numeric mode thresholds, dollar budgets, mixture weights, full taxonomy, statistical estimator, classifier architecture/weights, training examples and deployed release identity remain unavailable in the consulted sources. The described average-traffic optimization is **not** a per-task spending cap or an instruction to add one to Harness. The post's wish for more continuous adaptation is future intent, not proof of online learning on every request.

### Candidate pool, exclusions, toggles and pins

- The **specific current pool listed by the owning help page** is GPT-5.5, Claude Opus 5, Grok 4.6 and Claude Fable 5. It explicitly says a model can be directly selectable before Router validates it for routing; the pool changes over time. This is a dated documentary list, not a verified exhaustive runtime catalog. [S6]
- Historical/current inconsistency: July launch discusses GPT-5.6 Sol comparisons and Grok 4.5; August research describes Sol/Opus/Fable/Grok strengths and adding Opus 5; current pool names GPT-5.5/Grok 4.6. Do not infer hidden replacements, aliases, or that all models in pricing tables participate. [S3–S6]
- Enterprise blocked models are skipped for allowed alternatives. Current help requires Grok 4.6; blocking it disables Router. GPT-5.5 and Opus 5 are recommended; blocking both disables Router, blocking one reduces quality. Thus policy constraints can make routing unavailable, not just lower its score. [S1, S6]
- Team/group access merges by **most-permissive union**, not deny-wins. Teams set baseline access; organization groups widen it. These enterprise controls are not a promise that a single user's model-picker hide/show toggles restrict Auto. **No personal arbitrary Auto pool editor or per-model Auto pin contract was found.** [S7]
- Users steer modes, not an underlying model per Auto request. Admins can enable/disable Router, restrict up to two modes, control identity display and impose Auto: **Soft** defaults new chats; **Hard** locks the picker. Explicit model selection outside Auto exists; SDK advises a fixed model ID for reproducible comparisons. Neither proves immutable backend/release pinning. [S1, S2, S6]
- Availability also depends on plan, enterprise policy and provider region. Region docs say Auto selects an available model per request. These are eligibility constraints, not evidence of a particular outage-detection algorithm. [S6, S13]

### Latency and failure behavior

Speed is a documented objective and classification happens before expensive execution [S1, S3], but the consulted sources give **no classifier overhead distribution, request deadline, latency weighting or outage SLO**. Request complexity routing is not synonymous with latency-based failover.

Documented Router fallback is limited: skip blocked models for an allowlisted alternative where possible; disable Router when required coverage is missing; select available regional models. SDK documentation also distinguishes server-selected `auto` from explicit `auto-smart` and has separate transport/stall retry options. None specifies Router's invalid-classifier-output, timeout, provider-error or mid-response recovery contract. In particular, no public promise restores an exact parent model **and** thinking level. Pricing separately describes Fable security-guardrail rerouting to Opus; that is model-specific safety behavior, not a generic Router failure contract. Do not treat an execution failure as authority for Harness routing-failure fallback. [S1, S2, S6, S8, S13]

## Billing, accounting and visibility

- **Detailed current pricing reference:** Cost bills as a Cursor-model request from the weekly pool whatever model serves it. On Teams/Enterprise with Router, Balance/Intelligence bill at the routed model's list price: Cursor-model routes use the included pool, third-party routes use prepaid credit. Individual plans have Cost only. [S8]
- **Legacy plans remain different:** two monthly pools, Cursor Models and Other Models, and monthly on-demand usage rather than the new subscription's weekly pool/top-ups. A third-party Auto route uses Other Models at list price; named third-party subagents can do so even while the parent picker shows Auto. [S9, S14]
- Teams/Enterprise eligible third-party Balance/Intelligence routes can also incur the **Cursor Token Rate**, currently $0.25 per million input/output/cached tokens, on additional/BYOK usage. Cost and Cursor models are exempt; third-party models whose cache-read price is **strictly below $0.10/M across every pricing tier** are exempt. This is separate from model list price. [S8]
- **Unresolved documentation conflict:** Router help says “All Auto modes bill at the routed model's list price,” with similar language in the SDK and August guide, whereas detailed pricing, Router reference and available-models help explicitly exempt Cost from that accounting. The detailed pages are the more specific stated rule, **not a verified invoice**. Available-models help also says “no markup” generically despite the separately documented token fee. Confirm plan and terms before relying on those summaries. [S1, S2, S6, S8, S15]
- Routed identity is **hidden by default**; admins may display it at response start (reference) / in picker or request details after a turn (help). This applies to Balance/Intelligence. Public docs describe request-level cost in the usage dashboard and included usage/reset visibility. No reviewed page promises per-decision score, explanation, full eligible pool, or resolved model/effort provenance. [S1, S6, S8, S14]
- Interpretation: comparing token list prices alone misses context size, cache misses, output/reasoning usage, provider fees and included allowances. Catalog price, predicted spend, measured usage and actual plan billing are distinct quantities; Cursor's savings claims do not equate Harness subscription cost with API price. [S3, S5, S8]

## Evaluation evidence and its limits

- **July 22 launch:** claims classifier training on **600k+ live requests**, online A/B tests across millions of requests, satisfaction inferred from subsequent user behavior and generated-code keep rate. Reports Balance above Opus 4.8 satisfaction at about **36% lower cost**; Intelligence near Fable satisfaction at about **60% lower cost**. Early-access account savings use a same-traffic Opus-price counterfactual, not necessarily randomized account experiments. [S3]
- **August 6 update:** reports Balance above Opus 4.8 at **41% lower cost**, with 3% further satisfaction improvement; Intelligence above Fable-level satisfaction at **68% lower cost**. Compass's highest/lowest predicted-success groups reportedly receive positive signals **96%/71%** of the time. These are separately dated claims, not one pooled benchmark. [S5]
- Current help claims Intelligence delivers “about 20 to 30% higher quality” without a clear comparator or metric definition in that passage. It cannot be treated as an independently quantified intelligence score. [S16]
- Strength: real developer traffic and cache-inclusive cost are closer to agent work than a static standalone benchmark. Limits: correction/moving-on and keep-rate are proxies, not proof of correctness or user intent; account mix and task distribution affect results. The consulted sources do not supply enough raw data, confidence intervals, assignment details or reproducible artifacts to audit these percentages, generalize to Harness, or recover proprietary policy weights.
- No public production router implementation/weights were established by this research. SDK documentation describes access, not executable source for the classifier. No authenticated catalog or inference endpoint was called.

## Harness policy options/questions — not decisions

Existing accepted boundaries remain: local single-user factory; actual-parent inheritance; exact parent model/thinking fallback **only on routing failure**; enabled executable model/effort pairs; OpenRouter workers through factories with user enable/disable controls; browser PKCE; user-selectable Decider model/release. No arbitrary app concurrency cap or whole-spec time/spending cap is introduced. [HARNESS-8](https://app.plane.so/reilley/browse/HARNESS-8/) and the research brief own these boundaries, not Cursor.

1. **Meaning of balance:** choose between an adequacy-first, cost-conscious task policy and an evidence-backed task-class policy. Cursor's traffic-mixture budget is another design pattern, not available local evidence or an adopted budget. What establishes “adequate” and what happens when evidence is missing/stale?
2. **Decision surface:** assignment-time routing versus re-evaluation at explicit later boundaries raises context-transfer/cache costs and reproducibility questions. Cursor's per-request behavior does not decide Harness's unit of work.
3. **Evidence inputs:** task category, complexity, domain, recent outcomes and context size are useful candidate metadata. Which can Harness legitimately observe, which are required, and how is uncertainty recorded without pretending public benchmark rank predicts every task?
4. **Eligibility:** validate against the user's **enabled executable model/effort pairs**, not all catalog models or Cursor's pool. How are capability/region/provider availability snapshots refreshed and unsupported thinking choices rejected?
5. **Identity and visibility:** what selected-versus-resolved Decider release/alias, candidate snapshot, chosen pair, evidence date and routing-failure reason should be recorded? Hidden-by-default Cursor identity is a product choice, not a transparency requirement for Harness.
6. **Cost semantics:** separate catalog token rates/capacity from predicted and observed run usage/spend; account for cache switching and unknown pricing. What evidence is sufficient to call one adequate option cheaper without assuming subscription marginal cost equals API rates?
7. **Evaluation without new calls now:** what future authorized fixed-model baseline and task-outcome measures would test suitability, routing overhead and cost? Do not copy Cursor's percentages or statistical cutoff as locally validated thresholds.
8. **Failure distinction:** specify Decider timeout/invalid output/unavailable pair as routing failures with the already accepted exact-parent fallback. Worker execution failures remain in the accepted provider retry/admission contract, not an excuse to reroute. Cursor does not resolve the still-open Decider picker/release policy.

No option recommends calling a Cursor API, importing its model pool, acquiring credentials, or changing the accepted execution/authentication boundaries.

## Sources (all retrieved 2026-11-07; Cursor-owned)

- **S1 — Reference, undated:** [Cursor Router](https://cursor.com/docs/cursor-router.md), sections How it works, Optimization modes, Pricing, Team settings.
- **S2 — Reference, undated:** [TypeScript SDK](https://cursor.com/docs/sdk/typescript.md#cursor-router), Router, model parameters, per-run overrides, local retry options. SDK naming cited only as documentation, not an integration recommendation.
- **S3 — Launch post, July 22, 2026:** [Introducing Cursor Router](https://cursor.com/blog/router), Cursor Team.
- **S4 — Changelog, July 22, 2026:** [Cursor Router](https://cursor.com/changelog/router); feature-date version, no numeric client build.
- **S5 — Research post, August 6, 2026:** [How Cursor Router chooses the right model for the task](https://cursor.com/blog/how-cursor-router-works), Connor O'Keefe and Yuri Volkov.
- **S6 — Help, undated:** [Available models](https://cursor.com/help/models-and-usage/available-models.md), routing pool, visibility, Auto costs, switching and availability.
- **S7 — Enterprise reference, undated:** [Model and Integration Management](https://cursor.com/docs/enterprise/model-and-integration-management.md), model-access controls and team/group union.
- **S8 — Pricing reference, undated:** [Models & Pricing](https://cursor.com/docs/models-and-pricing.md), Auto modes, Cursor Token Rate, included usage and plans.
- **S9 — Legacy reference, undated:** [Legacy Cursor plans](https://cursor.com/docs/account/pricing/legacy-cursor-plans.md), usage pools and on-demand accounting.
- **S10 — Historical post, July 4, 2025:** [Clarifying our pricing](https://cursor.com/blog/june-2025-pricing), Michael Truell. URL month is not publication date.
- **S11 — Historical post, August 12, 2025:** [Updates to Teams and Auto](https://cursor.com/blog/aug-2025-pricing), Michael Truell.
- **S12 — Historical post, June 1, 2026:** [Improvements to Teams Pricing](https://cursor.com/blog/teams-pricing-june-2026), Cursor Team.
- **S13 — Reference, undated:** [Regions](https://cursor.com/docs/account/regions.md), regional eligibility and Auto.
- **S14 — Help, undated:** [Usage and limits](https://cursor.com/help/models-and-usage/usage-limits.md), Auto accounting, subagents and request-level costs.
- **S15 — Guide, August 12, 2026:** [Model routing: right model, right job, right price](https://cursor.com/guides/model-routing), restates launch measurements; retains older Grok 4.5/all-modes billing wording.
- **S16 — Help, undated:** [Cursor Router](https://cursor.com/help/models-and-usage/cursor-router.md), relaunch/default, goals and quality claim.

Source priority: mode/pool/pricing references own current documentary contracts; dated posts own historical statements and reported experiments. Neither establishes verified account access or exact present operational behavior. Quotation marks identify exact source wording; design questions and explicitly labeled interpretations are not Cursor promises.
