# Safe compaction and context management

Research for [HARNESS-47](https://app.plane.so/reilley/browse/HARNESS-47/). Retrieved **2026-11-07 UTC**. Research only: no Harness production changes, inference experiments, authentication, paid requests, PR, merge, tracker mutation, or resource cleanup. Findings below are provisional design advice, not an amendment or implementation authorization.

## Decision summary

**Keep authority in durable state; treat compacted context as a disposable, lossy view.** Start with selective context construction, source-linked artifacts, recent complete tool exchanges, and an explicit context-exhausted outcome. Consider deterministic old-observation masking before adding another model call. Evaluate a bounded structured checkpoint only behind the ordinary execution/admission gates. Do not make native provider compaction, a trained compressor, recursive inference, or a semantic-memory service an MVP prerequisite.

The decisive distinction is not “summary versus no summary.” It is **reversible model-context reduction versus destruction of evidence or reconstruction of authority from prose**. Compaction cannot authorize continuation, declare a child settled, restore an allowance, reconcile an uncertain mutation, or make an unmerged PR complete.

| Option | Recommendation | Why / adoption condition |
| --- | --- | --- |
| Pinned instructions + current durable-state projection + on-demand source-linked artifacts | **Recommend** | Fits the approved contract; useful without another inference call. Preserve provenance and trust boundaries. |
| Reversible masking of older bulky tool observations, retaining IDs, outcomes and artifact references | **Conditional, first compression candidate** | Cheap and auditable; empirical quality loss is still possible. Preserve complete call/result structure and full original evidence outside the rendered prompt. |
| Structured checkpoint + recent tail, produced by the already selected eligible model | **Conditional** | Ordinary cross-provider text generation is more portable than native opaque state. Needs matched quality evaluation, conservative installation and actual usage accounting. Not a durable authority record. |
| Native OpenAI / OpenRouter compaction | **Conditional, later** | Exact endpoint/auth/model capability is not inherited from Responses compatibility. Resolve subscription entitlement and opaque-item replay before adoption; fail visibly rather than reroute. |
| Background speculative compaction | **Defer** | Adds stale-candidate, cancellation, late-cost and installation races. Foreground checkpointing should be correct first. |
| Hierarchical/folded task-local summaries and artifact retrieval | **Conditional, later** | Promising for naturally bounded subtasks; use existing explicit assignments/joins, not an alternative completion or permission protocol. |
| ACE-style evolving playbooks, recursive LM environments, trained compressors | **Defer** | Extra inference/training/state architecture and evaluation obligations; reported gains do not establish this factory's safety or cost. |
| Destructive transcript deletion, summary-owned approvals/allowances, hidden model fallback or post-Stop compaction | **Reject** | Violates the governing contract even if compression quality is high. |

No option is demonstrated superior on Harness. Source inspection establishes implementation behavior, not comparative efficacy or provider entitlement.

## 1. Scope, baseline and evidence discipline

### Governing inputs

The immutable input is `/tmp/harness-mvp-implementation/approved-spec.html`, SHA-256 `a70f27d50c2b69eb87e3c1b28f61ac1138ef4400e0f1d1a3ec033d65b86eb6bc`, with `/tmp/harness-mvp-implementation/spec-comments.json`, SHA-256 `35d435510a6ff9157bd4a2fe2238cd3eef80f28fb42674f86644fc4db87478ec`. [HARNESS-19](https://app.plane.so/reilley/browse/HARNESS-19/) is canonical; it was retrieved read-only as a cross-check, not used to silently replace the approved local snapshot. Relevant sections: durable agents/runs/coordination; instruction snapshots; provider admission/accounting; steering/Stop/recovery; exact review/integration/merge; verification V04, V08–V15, V17–V18.

Preserve these invariants across every reduction:

- Authoritative scope/version and explicit approvals; durable agent/assignment/run identity; selected instructions and provider/model/effort; steering order/generation; captured joins and child outcomes; dispatch intent and deduplication; uncertain/completed effects; allowances; review, readiness and authorized merge evidence remain **outside summaries**.
- Rehydrate from trusted current state, pinned instructions and source-linked artifacts. A quoted summary, tool result, `DONE`, model final response, ticket closure, or ready PR cannot grant authority or settle a spec. Only independently verified authorized merge settles it.
- Stop, input/approval waits, restart/explicit Resume, model eligibility and provider admission gates apply to **summarization inference too**. A timer, restart, summary installation or child result cannot bypass them. No widened permissions, rerouting, blind replay, or accounting/rework reset.
- No inference polling, global/per-role slots, whole-spec spending/time caps, application Decider deadline, separate semantic-memory service, permanent test dependencies, private debug APIs, or destructive transcript deletion.
- OpenRouter workers are approved future work, **not implemented on the inspected baseline**. Do not turn this report into another direct-provider integration or modify personal/global configuration.

### Actual Harness baseline

Pinned baseline: [`6b4512d6ef7b0aa38783ff939bfd0f76234f41a4`](https://github.com/Reilley64/harness/tree/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4).

| Inspected behavior | Evidence / implication |
| --- | --- |
| Stateless OpenAI subscription-token Responses worker, `store:false`, `stream:true`, encrypted reasoning requested | [H1]. No implemented automatic compaction policy or portable compaction item in the audited adapter. Account credentials are not a capability test. |
| Private continuation holds tool calls/reasoning; reasoning filtered when model identity changes | [H2]. A summary must not expose or synthesize private reasoning or forge provider items. Tool protocol validity is separate from readable prose. |
| Ordered steering/cancellation around model and tool boundaries, generation checkpoint before inference | [H3]. Reuse the owning application lifecycle; do not bolt on an independent compaction loop. |
| Conversation persistence bounded at 16 MiB; provider continuation has its own 1 MiB bound | [H2], [H4]. Disk-record bytes are not context tokens. Context compression alone does not solve append-only storage capacity. |
| Baseline has a 1,200-second turn timeout and `MAX_MODEL_STEPS` / `MAX_TOOL_EXECUTIONS` | [H3]. These are baseline facts, **not recommendations to retain them** or permission to impose a Decider deadline or whole-spec limits. Approved future contracts govern implementation. |
| Usage adapter exposes reported input/output tokens, not a complete cache/cost breakdown | [H5]. More granular native/summary usage cannot be claimed implemented. Unknown price/cache fields must stay unknown. |

### What counts as evidence

Open source is pinned to a commit, not a moving branch. Closed-client documentation is labeled documentation, not runtime proof. API schema types establish a documented wire surface, not successful execution on every route. Papers are original author publications, versioned below; their figures are not independent replications. No tests or benchmarks of those projects were executed. Public HTTP fetches, local text extraction, source outlining/search, and repository/citation checks were the only research operations.

Retained evidence: `/tmp/harness-compaction-sources/` (fetch ledger, official documentation snapshots, papers, author-code README pins, canonical issue snapshots, validation output) and `/tmp/harness-completion-sources/` (reused commit-pinned source, `meta.json`, `tree.json`). The latter's four primary pins were rechecked with the GitHub commit API. `citation-check.json` records source paths/ranges and hashes; `report-validation.json` records final reference checks. `evidence-inventory.json` inventories retained source files by SHA-256. The locally installed Pi package is **1.5.0**; its docs were inspected for orientation, but implementation claims below refer to the separately pinned public commit, not that installed release. Failed/redirected URLs and unavailable internals are limits, not evidence of absence.

## 2. Actual implementations

### Version and provenance inventory

| Product | Audited version | What this proves |
| --- | --- | --- |
| Codex | `openai/codex@d4a341bd52cc47ff635761334bde8a972d6de943`, 2026-11-07 | Rust runtime source, including newer remote v2 and local paths; not a release-wide entitlement promise. |
| Pi | `earendil-works/pi@085c84e98e7f4c950f2be606466fff4f75d8fe7c`, 2026-11-06 | Core runtime compaction, persisted projection/recovery, extension seam and docs. |
| OpenCode | `anomalyco/opencode@8bad789e1181aedb57327f41a2e24c4d710763d5`, 2026-11-07 | Core compaction service, native adapter utility and configuration docs; provider route capability remains a separate question. |
| Gemini CLI | `google-gemini/gemini-cli@62bab5b232c15b925d31e655ce844ab939c882b7`, 2026-11-06 | The standard client's `ChatCompressionService` path and related source. Other context-graph/compression services coexist; this is not a claim that every experimental pipeline uses this exact path. |
| Claude Code | Official docs retrieved 2026-11-07; version qualifications where stated | Documented closed-product behavior; no access to its current implementation. Public changelog provides bug reports, not proof that all edge cases are solved. |
| Cursor | Official context/Agent docs retrieved 2026-11-07 | Documented context attachment and side chats only for the pages inspected; internal compaction policy remains unestablished. |

### Trigger, budget and summary comparison

| Runtime | Trigger / budget | Reduction, summary and model | Important distinction |
| --- | --- | --- | --- |
| **Codex** | Manual compact task; pre-sampling, context/budget and post-turn paths. Token-budget mode, session versus window scope, usable full-context limit and asynchronous thresholds are distinct. A model/compaction-compatibility change can require previous-model compaction. | Local summary prompt or provider-capability-selected **remote v2**. The audited v2 sends `compaction_trigger` through streamed Responses with compaction metadata; it is **not simply an invocation of `/responses/compact`**. Local oversized summaries retry after dropping oldest material. | Remote capability is explicit. Prior/current-model retry exists in Codex; **do not import that fallback into Harness**. [C1], [C2], [C3], [C4] |
| **Pi** | Automatic checks against `contextWindow - reserveTokens`, manual `/compact`, and one compact/retry recovery for selected overflow or early final length failures. Defaults: reserve 16,384 and recent 20,000 tokens, with per-model overrides. Checks include canonical projected history after tools, before the next response; not just after an assistant reply. | Structured Goal / Constraints & Preferences / Progress / Key Decisions / Next Steps / Critical Context; update previous summary. Split-turn prefix gets a distinct checkpoint. Core uses captured current model/settings; extension can supply its own result or another summarizer. | These are Pi's policies, not safe universal thresholds. Native runtime compaction and custom-compaction extension behavior are separate. [P1], [P2], [P3], [P4] |
| **OpenCode** | `compaction.auto`, `keep.tokens` and `buffer`; manual, auto and one overflow-recovery attempt. Default keep is 15,000 tokens and free buffer 15% (at least 16k). Experimental background precomputation is possible. Native is explicitly selected through provider/model `settings.compaction`. | Local prompt with summary + retained tail; staged transcript shrinking if summary input is too large; `agents.compaction.model` can override session model, falling back to session model if unresolved. Native replacement may carry encrypted state. | Buffer is room kept **free**, not the trigger percentage. Local summary, experimental precomputation, native endpoint and hooks are distinct. OpenCode's model fallback is not suitable for automatic import into Harness. [O1], [O2], [O3], [O4] |
| **Claude Code** | `/compact [focus]`, automatic compaction and per-model `/autocompact` window. Current docs describe model-specific boundaries, not one universal historical “95%” rule. | Structured conversation summary; pruning/clearing older tool results also affects context. Since v2.1.198, summary request inherits session extended-thinking configuration. Root guidance reloads; some scoped/history-loaded rules and hook content are only summarized. | Manual rewind/partial summarization and idle compaction are additional documented features. No inference about its hidden prompt, exact tool-pair algorithm or transactional recovery. [A1], [A2], [A3], [A4] |
| **Gemini CLI** | Standard client calls `ChatCompressionService`; default trigger fraction **0.5**, configurable, with forced compression. Older function responses can collapse first; three recent tool-response turns are protected, with retrieval exemptions/budgets. | Model-config alias chosen from current model; summary then verification call; token count checks reject invalid/empty or larger replacements. Recent history retained; boundary logic avoids splitting function-call responses incorrectly. | Two model calls and preliminary truncation have costs/losses. A separate `ContextCompressionService` also exists; its Flash-Lite batch code must not be described as the standard client's sole summarizer. [G1], [G2], [G3] |
| **Cursor** | Inspected official pages document `@` attachment and durable side chats referencing a parent context. | Compaction thresholds, model, summary format, tool/opaque-item rules, retry and cache accounting were not established. | A former summarization URL redirected to Agent overview; guessed context paths 404ed. Neither means the product lacks compaction. [U1] |

### Integrity, recovery and repeated-compaction behavior

**Codex.** Local compaction constructs a fresh context from retained user messages and summary, trims to budget, retains selected tool outputs and reinjects context. That creates a distinct drift/loss surface from opaque native compaction. Remote v2 prepares ordinary instructions/tools/cache controls, streams compaction output, and installs a validated candidate. Background state captures history version, model/auth ownership and context; stale/changed candidates cannot simply overwrite newer work. Cancellation and history checks are valuable patterns, not evidence of external-effect rollback. Full rollout history versus active model history must be distinguished. Compaction usage/context accounting is not a reason to reset run/spec spend. [C2], [C3], [C4], [C5], [C6]

**Pi.** Core compaction operates over canonical session projection, keeps a recent boundary by entry ID, handles split turns, updates the prior summary and records usage/file references. Serialization explicitly omits thinking and truncates each tool-result representation to **2,000 characters**; this saves prompt space but can omit the only decisive diagnostic. Recovery uses persisted `context_edit` omissions rather than deleting the recorded attempt; a failed length response with tool calls retains synthetic failed results and follows the scheduler. Abort signals and terminal extension events exist, while operation model/settings are captured. These are runtime protections, not a claim of error-free repeated summaries or correct preservation of external approvals. [P1], [P2], [P3], [P4]

**OpenCode.** Precomputed summaries require matching history and tail budget and are not reused for overflow/native mode; unusable background work is interrupted. Native encrypted state cannot be estimated like ordinary text, so measured usage matters. Repeated local compaction includes the rendered previous summary: **summary-of-summary drift remains possible**. The stream rejects length-limited, filtered, unknown/incomplete and error finishes as unusable. Each completed model call is billed even if compaction later fails/interruption occurs; background totals are carried separately. Hooks can directly provide a summary and optional usage, which is an extension trust boundary, not free native computation. [O2], [O3], [O4]

**Claude Code.** Docs explicitly warn that early conversation instructions can be lost and that repeatedly refilling from a giant file/tool result eventually produces a thrashing error rather than an endless loop. Root guidance versus scoped/history-loaded guidance has different survival semantics. The dated public changelog records fixes involving stale/incorrect idle summaries, loss of previous summary under input shortening, and parallel-tool/forked-conversation losses. Those are useful failure hypotheses, **not controlled failure-rate measurements**. No assertion is made that cancellation/restart/model-switch/install semantics match the open implementations. [A1], [A2], [A3], [A4], [A5]

**Gemini CLI.** The inspected standard service checks whether compression actually reduces size and retains a tail. A second summary-verification call can improve a checkpoint but cannot establish fact preservation. Its abort signal is caller-provided with a fallback controller; source alone does not prove the complete Stop/restart contract. The separate compression service includes explicit function-call/result adjacency validation and an uncompressed fallback. Do not transplant its fixed summarizer model or claim that fallback is permission to dispatch an ineligible model. [G1], [G2], [G3]

**Across clients:** readable summaries, native encrypted compaction, observation pruning, extension hooks and manual file notes solve different problems. None of the inspected materials proves that repeated compaction preserves all task-relevant evidence, that a summary can resolve uncertain writes, or that model switching can safely reuse every opaque item. Keep raw evidence and state provenance; independently verify important assertions at use time.

### Usage/cache/cost consequences

- A summary request reads old history and emits new tokens; overflow retries, verification and discarded background summaries add cost. Count attempts actually sent, including failure/cancellation and late usage; waiting is not an attempt. Do not double-count when background usage is later attached to a checkpoint.
- Rewriting a prefix can invalidate cached input. Claude's docs describe cache rebuilds after compaction and the separate cost of reading history to summarize it. Shorter *raw* prompts do not imply lower *billed* cost. [A4]
- Pi combines summary usage, including separate history/prefix calls; OpenCode accounts per finished call. Harness baseline's input/output-only usage is insufficient to report all these dimensions faithfully. [P2], [O3], [H5]
- Subscription usage, API billing and OpenRouter credits are different accounting regimes. Record reported tokens, cache-read/write fields, estimates and unknown costs separately. Never convert undocumented subscription “cost” into fabricated dollars.

## 3. Provider/auth compatibility: do not transfer claims across products

| Path | Positive primary evidence | Not established / safe decision |
| --- | --- | --- |
| OpenAI API-key Responses compaction | Official guide documents server-side `context_management` compaction and standalone `/responses/compact`; encrypted compaction items are opaque replay state, not human summaries. API reference has usage. [N1], [N2] | Per-model support, exact instruction/tool handling and accounting still need the selected model's documented contract and an authorized live probe. Not all Responses-compatible endpoints implement this. |
| **Sign in with ChatGPT subscription** via public OpenAI API | Official SIWC guide prescribes OAuth bearer, account catalog, `https://api.openai.com/v1/responses`, `store:false`, `stream:true`; it explicitly rejects using the SIWC flow against ChatGPT `backend-api` endpoints. [N3] | Inspected SIWC guide/reference do **not explicitly grant native compaction entitlement or define the compaction endpoint/trigger contract for this auth**. General API compaction docs plus subscription inference support are insufficient. Status: **unverified, not proven unsupported**. Require an explicit official auth/model statement or separately authorized minimal probe before enabling. |
| Codex's own ChatGPT/provider integration | Audited provider capability routes OpenAI remote v2 through Responses compaction triggers; local path exists for unsupported providers. [C1], [C2], [C3], [C4] | This proves that Codex implements the protocol, not that another client using SIWC on the public API has the same entitlement, headers, model program or opaque-item compatibility. Do not copy a private endpoint or claim parity. |
| OpenRouter Responses | Official guide states stateless-only: `store:true` or non-null `previous_response_id` returns 400. **Published OpenAPI explicitly includes `CompactionTriggerItem`, encrypted `CompactionItem`, and `OutputCompactionItem`**, reachable from `ResponsesRequest.input`; this is stronger evidence than mere compatibility. [R1], [R2] | The inspected docs/schema do not supply a tested provider/model allowlist or a native-compaction behavioral/usage guarantee for the planned worker. No standalone `/responses/compact` path appears in that schema. Status: **schema-level trigger/replay support; operational route support unverified**. Never infer universal support or categorical absence. |
| OpenRouter Anthropic Messages surface | Published schema contains `context_management.edits` including `compact_20260112`, compaction content blocks, stop reason and usage iterations. [R2] | A distinct protocol surface, not automatic capability of an OpenRouter Responses worker or approval for another direct integration. Beta/model/route behavior remains to be verified. |
| OpenRouter transforms / compression middleware | Middle-out transforms remove/reorder context; the ScaleDown cookbook is a separately invoked compression pipeline. Schema also exposes a `context-compression` plugin. [R2], [R3], [R4] | These are not proof of provider-native state compaction, preservation of tool pairs, or subscription support. Do not silently enable them under the selected-model contract. |
| Application-owned text checkpoint | Ordinary generation can summarize a supplied text view using the exact admitted provider/model/effort | Broad potential portability, **not demonstrated compatibility** with every model. Enforce size, structured-output/tool restrictions, gates and accounting; never use another model merely because it is cheaper. |

**Result of the requested subscription investigation:** no authorized authenticated probe was performed and no official SIWC-specific native-compaction guarantee was found in the inspected sources. Do not promise this feature for the existing subscription path. Retain an explicit unknown capability and a visible context-exhausted/configuration outcome. “Codex does it” is not sufficient evidence.

For a future live capability check, pin endpoint, auth class (never token), account model catalog revision, requested/resolved model/effort, permitted parameters, route restrictions, HTTP status/request ID, returned item types, replay acceptance and usage. A successful empty/light request is not enough: compact a benign complete tool exchange, replay the opaque item with the same admitted identity, and test documented rejection without invoking real tools. Consent and model/provider gates are required before each actual call. Do not run this plan as part of research.

## 4. Recent original research: methods **and** evidence

The search covered original publications through retrieval date, with emphasis on August 2025–November 2026. This is a targeted primary-source review, not a systematic-review claim. Recent November preprints are especially provisional. Numbers below are **authors' reported results**, not Harness measurements; percentage points, token counts, benchmark scores and dollars are intentionally not conflated.

### Paper inventory and reproducibility

| Work, exact version and authors | Method / prerequisites | Original implementation status |
| --- | --- | --- |
| **The Complexity Trap: Simple Observation Masking Is as Efficient as LLM Summarization for Agent Context Management**, [2508.21433v3](https://arxiv.org/html/2508.21433v3), 2025-10-27 (v1 2025-08-29). Tobias Lindenbauer, Igor Slinko, Ludwig Felder, Egor Bogomolov, Yaroslav Zharov. | Keep reasoning/actions but mask old observations; compare rolling LLM summaries and a hybrid. Runtime-only context view, not model training. | Authors' [code](https://github.com/JetBrains-Research/the-complexity-trap/tree/bf15b5fb7d279679035a007ac9a81084d6b9a89a). SWE-agent configs and evaluation/analysis instructions; original provider/model availability still matters. |
| **Scaling Long-Horizon LLM Agent via Context-Folding**, [2510.11967v1](https://arxiv.org/abs/2510.11967v1), submitted 2025-10-13 (retained PDF dated 2025-10-15). Weiwei Sun, Miao Lu, Zhan Ling, Kang Liu, Xuesong Yao, Yiming Yang, Jiecao Chen. | Active branch/return environment with summaries plus **FoldGRPO training** of Seed-OSS-36B; 32,768 active context. Prompt-only folding is an ablation, not the whole method. | Authors' [FoldAgent](https://github.com/sunnweiwei/FoldAgent/tree/58a2d6964ecebe99940529eace50a0558901b8a5); paper/project link. PDF retained. Do not assign trained-policy gains to an ordinary tool wrapper. |
| **Agentic Context Engineering: Evolving Contexts for Self-Improving Language Models**, [2510.04618v3](https://arxiv.org/html/2510.04618v3), 2026-03-29 (v1 2025-10-06). Qizheng Zhang, Changran Hu, Shubhangi Upasani, Boyuan Ma, Fenglu Hong, Vamsidhar Kamanuru, Jay Rainton, Chen Wu, Mengmeng Ji, Hanchen Li, Urmish Thakker, James Zou, Kunle Olukotun. | Generator/reflector/curator create incremental playbook entries, avoiding monolithic rewrite/collapse; offline and online adaptation consume feedback and extra inference. | Authors' [ACE](https://github.com/ace-agent/ace/tree/82709de050e1db6e6ef2f07bcb0393560b94992a). A mutable learned playbook is advisory content, not permission or repository policy. |
| **Recursive Language Models**, [2512.24601v3](https://arxiv.org/html/2512.24601v3), 2026-05-11 (v1 2025-12-31). Alex L. Zhang, Tim Kraska, Omar Khattab. | Prompt lives in an external REPL/environment; root programs inspections and recursive model calls. Main GPT-5 setup uses medium reasoning at root and GPT-5-mini subcalls; its compaction comparator uses GPT-5-nano summaries. V3 also studies trained RLM-Qwen3-8B. Not simply compressing the same transcript. | Authors' [RLM](https://github.com/alexzhang13/rlm/tree/c4cfe00266c890bdf83ca293ee2c1f67e1d3bffd); distinct environment, call tree and accounting needed. |
| **ReFold: Training-Free Reversible Inter-Turn Context Folding for Long-Horizon Agents**, [2610.07863v1](https://arxiv.org/html/2610.07863v1), 2026-10-06. Yupeng Su, Jiayi Tian, Zheng Zhang, Souvik Kundu. | Render-layer duplicate observations and model-marked folding, reversible restore, append-only source trajectory; chunking to preserve cache reuse. | No original runnable repository/weights link was established in the inspected paper links. Paper-level reproducibility only here. |
| **Spine: Compiling Subtask Trees into Structured Context for Long-Horizon Coding Agents**, [2610.37224v1](https://arxiv.org/html/2610.37224v1), 2026-10-27. Jiahong Xiang, Kunqiu Chen, Lingming Zhang, Yuqun Zhang. | Agent-maintained nested subtask tree and success/failure boundaries fold resolved segments into a task spine. Optional Codex/Pi integrations, **not built-in behavior of those clients**. | Authors' [SpineCodex](https://github.com/GhabiX/SpineCodex/tree/bd1926e2417eb6e17722c372455dce9be8355340) and [spine-plugin](https://github.com/GhabiX/spine-plugin/tree/a84bcf7a8c2f8eac0962c4dc47914cdf344c57b1). Requires control tools/working-memory state. |
| **SUMONE: A Simple and Efficient Summarizer for Plug-and-Play Agent Memory**, [2611.06445v1](https://arxiv.org/html/2611.06445v1), 2026-11-04. Jizhe Zhang, Ang Li, Chenheng Zhang, Yisen Wang. | Dedicated Qwen3.5-2B summarizer, reconstruction reward and GRPO; target-domain evaluation and trained checkpoint required. | Authors state code/implementation resources **will be released upon acceptance**; no released checkpoint was established at retrieval. “Plug-and-play” does not mean weights are available. |
| **What Is This Memory Useful For? Measuring Task-Conditioned Utility in Long-Term Agent Memory**, [2611.03930v1](https://arxiv.org/html/2611.03930v1), 2026-11-02. Chih-Hsuan Yang, Anjir Ahmed Chowdhury, Feng Yan, Rajeev Thakur. | Paired later-decision utility of raw/summary/structured memories with bearing versus control prior evidence. Tests evidential usefulness rather than agent task completion. Its answer-bearing screen uses gold answers and is an evaluation instrument, not a deployable retriever. | Authors' [ConditionalMemory](https://github.com/ChihHsuan-Yang/AAMAS2027-ConditionalMemory/tree/386ac65a8b5fc46414abdd2755375f2a52d4d3b8); supplementary notebooks/configs. Unequal lengths are an explicit confound. |

**Publication status:** the evidence reviewed above is the exact versioned arXiv manuscript, not automatically a peer-reviewed proceedings version. The [Complexity Trap abstract](https://arxiv.org/abs/2508.21433v3) identifies a DL4C camera-ready workshop version co-located with NeurIPS 2025, not a NeurIPS main-track paper; the [ACE abstract](https://arxiv.org/abs/2510.04618v3) identifies ICLR 2026. These are author-supplied publication comments, not independently checked venue decisions. The other six inspected abstracts do not establish a peer-reviewed venue; publication elsewhere remains unknown. No author-reported result here was independently reproduced.

README/code pins above are later repository snapshots, **not asserted to be the exact commits used to generate paper tables**. They were inspected for availability/instructions, not executed or exhaustively audited. Mutable project claims and dependency access can still block replication.

### Evaluation results and limitations that affect a decision

| Work | Matched evidence / units | What not to conclude |
| --- | --- | --- |
| Complexity Trap | Main SWE-agent study uses **500 common SWE-bench Verified issues** per model, five model configurations; Table 1 reports solved percentage, **cost per instance** and bootstrap intervals, not three-seed averages. Gemini 2.5 Flash thinking: raw 40.4% / $0.56, masking 36.4% / $0.24, summary 31.4% / $0.25. Qwen3-Coder-480B: raw 53.4% / $1.29, masking 54.8% / $0.61, summary 53.8% / $0.64. Summary trajectories can become longer; hybrid is separately evaluated. OpenHands generalization uses **one model configuration, Gemini 2.5 Flash without thinking, on 50 issues**, not the full five-configuration matrix; its masking window requires scaffold-specific tuning. | Masking is not universally quality-preserving: Flash loses **4.0 percentage points**, summary 9.0, versus raw. Costs/instance are not cost/solved issue. GPU-hosted Qwen serving and API-priced Gemini are different cost bases. The critic-enhanced summary study reports no solve-rate improvement and worsened trajectory elongation, but its denominator is internally inconsistent: Appendix D's setup names SWE-bench Lite-50 while D.3/Figure 7 name 150 SWE-bench Verified issues. Do not reconcile that by assumption. No Harness effect-safety evidence. |
| Context-Folding | Seed-OSS-36B, 32K active context and at most ten branches; BrowseComp-Plus **150 evaluation queries** (680 training), SWE-bench Verified **500 issues**. Table 1 Pass@1: trained FoldGRPO .620 / .580, prompt-only folding .420 / .492; 327K ReAct .478 / .552; 32K ReAct .286 / .436; 32K summary with ten sessions .386 / .488. Greedy decoding; table also reports tool calls, e.g. SWE FoldGRPO 96.5 versus long-context ReAct 49.5. The table's 32K×10 is a maximum token-budget column, not measured cumulative API input. | The gain is not evidence of lower cost or prompt-only portability. Training and longer/more tool-intensive behavior are part of the comparison; active window size and maximum branch budget are not total billed tokens. Baselines with different windows need separate interpretation. |
| ACE | AppWorld (TGC/SGC on test-normal/challenge), FiNER and Formula (accuracy), plus other domains. AppWorld Table 1, DeepSeek-V3.1: online ACE average **59.5** versus ReAct **42.4**, a **17.1-point** difference in the average of four metrics, not a task-count denominator. Official splits are used; numerical evaluated sample counts are not supplied in that main table. The headline 10.6% / 8.6% averages are aggregates over selected comparisons, not one paired success rate. In GPT-5.1 caching appendix, **91.8% of evaluation input tokens** cached and **82.6% billed-input reduction relative to raw-token accounting**. | Aggregate improvements are not a common unit across all tasks. Extra generator/reflector/curator work, feedback access and environment-specific playbooks must be counted. Cache illustration is not a universal reduction against the same matched baseline/provider. Do not install learned instructions as trusted policy. |
| RLM | Main Table 1 covers CodeQA (sample count not stated in the inspected main task description), BrowseComp-Plus **150 sampled queries** with 1,000 documents each, OOLONG **50 tasks**, OOLONG-Pairs **20 tasks**. S-NIAH **50 tasks** is an additional context-scaling probe. Context reaches millions of tokens and differs by family. Table 1 reports **mean API cost ± standard deviation**; appendix distributions/quantiles are separate. Recursive calls improve some tasks; an ablation shows deeper recursion can hurt Qwen3-Coder through syntax/runtime errors. | Not a 150-task universal agent benchmark and not “infinite context for free.” Runtime, subcalls, tail latency and environment failures are material; external prompt access is an architectural prerequisite. |
| ReFold | **50 tasks each** from SWE-bench Verified, SWE-bench Pro, Multi-SWE-bench, Terminal-Bench 1.0 and 2.1, with Qwen3.6-27B and Qwen3.6-35B-A3B in mini-swe-agent/vLLM (two RTX 6000 Ada GPUs/model). Main tables compare success, tokens, latency and estimated dollar cost; most comparisons use 50 paired tasks, while the saturated concurrency w=64 experiment uses **71**, not 50. Cache/chunk illustration: **one trajectory**, prefix reuse 51% rerender-every-step, 87% chunked, 84% base. Chunk ablation is **20 tasks**. | One-trajectory cache ratios are not benchmark-wide billing results. Chunk size trades compression/adaptivity against stable prefixes. Restore capability does not establish that the agent asks for missing evidence. The paper also permits **re-executing an original command** to refresh an observation: that is not safe recovery for uncertain mutations in Harness; use retained-artifact restore/read-back, never blind replay. |
| Spine | Table 3 SWE-Milestone, **98 milestones**, GPT-5.6 Sol: native Codex score 38.1, **14/98 resolved**, $681.22; Spine score 42.6, **19/98**, $460.77. ProgramBench uses **50 sampled tasks** out of 200: average pass score 62.55→73.35 but **total cost rises** $210.21→$221.56. FrontierSWE is a separate ten-task subset with five trials per task; seven of 17 tasks were unavailable. | Milestone score is not resolved percentage; dollars are benchmark totals, not per-query. Task sampling, unavailable environments, proprietary model availability and integration choices limit inference; benchmark-specific improvement is not a universal cost advantage. |
| SumOne | DeepSearchQA and SWE-bench Verified; **actual evaluated sample counts are not explicit in the inspected setup/main table**, so benchmark universe sizes are not substituted. Qwen3.5-35B-A3B / OpenCode main table: full-history score .52 / **estimated $0.170 per query** versus SumOne .48 / $0.038; coding .63 / **estimated $0.286 per issue** versus .59 / $0.071. Includes recency truncation, self-summary, prompted 2B and CoMem baselines; native harness compaction disabled. | Each comparison loses **0.04 absolute score**. Dollar figures use token/pricing estimates, not audited invoices. The prose says coding uses 58% of full-history cost, but Table 1's .071/.286 is about 25%; retain this internal inconsistency rather than reconcile it by assumption. Missing sample counts, training/checkpoint availability and backbone dependence prevent immediate adoption. |
| What Is This Memory Useful For? | Main paired analysis uses **277 shared cells**, not 277 complete agent runs. Bearing-minus-control mean utility: raw **+.3174** (CI excludes zero), summary **+.0613**, structured **−.0498** (both CIs include zero). Mean lengths raw **9,228**, summary **323**, structured **238 tokens**; content-retention follow-ups expose what compression lost. | Not a proof that every summary is useless, nor a completion benchmark. Fixed-total-context and content-retention follow-ups partly address length confounding; position/session/reader changes alter some labels. The tested SWE/scientific-coding transfer shows no measurable benefit even for raw memory. Later evidential usefulness needs its own oracle. |

The most transferable recent result is **methodological**: evaluate task outcome, later evidence usefulness, cost and failure behavior together. Do not select a method because a shorter prompt looks efficient or a paper's best aggregate improves. Most papers do not test approval integrity, Stop, unknown external effects, concurrent joins or restart safety at all.

## 5. Narrow design options for Harness

### MVP boundary: preserve the approved architecture

This section proposes reviewable choices; it does not add another MVP requirement. Required durable state, instruction snapshots, event delivery, explicit Resume, provider eligibility and merge authority already come from HARNESS-19. A compactor must consume those services, not create substitutes.

1. **Context construction before compression.** Load only applicable root/phase guidance and requested scoped references. Build a small trusted current-state projection from durable records; keep quoted historical/task/tool/artifact content source-labeled and separate from authenticated user steering and approved authoritative records. Store large observations as immutable or revision-linked artifacts and render an excerpt plus reference, not a fabricated result. Use ordinary authorized reads for retrieval; no semantic-memory service.
2. **Protect protocol islands.** Keep recent complete assistant tool-call/result exchanges, stable call IDs and actual success/failure/unknown outcomes. Never leave an orphan result, invent a success, drop an unresolved mutation, or summarize opaque provider state as if it were a valid replay item. Preserve exact source separately. If a single required exchange cannot fit, stop with an actionable context/configuration outcome.
3. **Prefer a reversible deterministic view first.** Candidate A masks old bulky observations with artifact ID, tool/call identity, actual disposition and retrieval hint; keeps recent full exchanges and selected evidence. It must not delete durable raw history. Adopt only if matched evaluation shows acceptable later evidence use and task outcomes. If persistence capacity is reached, fail conservatively or separately design authorized archival—do not quietly discard records.
4. **Optional structured checkpoint.** Candidate B stores a versioned derived artifact: source range/hash, captured model/instruction/generation references, summary text, artifact/evidence links and usage. Summarize working hypotheses, decisions and next investigative questions, explicitly marking uncertainty. Never serialize secrets or private chain-of-thought, and never put approvals, allowances, joins or effect disposition solely in that text. Schema/length/provenance validation detects malformed output, **not semantic truth**.
5. **No autonomous summarizer loop.** A summary is an ordinary model attempt admitted by current eligibility, Stop/input/approval/restart/provider gates. Use the selected eligible model/effort unless a human approves an explicit different arrangement; no cheap-model default, hidden gateway fallback or extra Decider routing. Account all attempts under the real owner/run/spec and existing retry/allowance rules; do not invent a new retry pool.
6. **Visible exhaustion is acceptable.** If safe reduction fails, cannot fit required state, or needs unavailable capability, preserve the old view and evidence; surface the reason and allowed human choices. Do not turn failure into implicit completion, relaxed scope or unauthorized fresh inference.

### Candidate installation and recovery contract

A foreground candidate is simpler than speculative background compaction, but both need the same safety checks:

- Capture source-history revision/range, current controlling run/generation, instruction snapshot, selected provider/model/effort/auth ownership, and state-projection revision **before** inference. Record actual sent attempt and known cost without exposing credentials.
- Keep a candidate derived and uncommitted while the call runs. Cancellation or failure cannot modify the active prompt view or replay an external tool. Preserve any completed/uncertain effects and late usage independently.
- At a safe continuation boundary, re-read durable state and check gates/ownership/generation/revisions. If relevant state changed (steering, child completion, instructions, model, Stop, wait, restart), discard or rebuild the candidate; never install it over new facts. Do not hold a storage lock over model execution.
- Atomically checkpoint the selected derived-view reference plus provenance under checked revisions; raw history is still retained. Rehydrate current authoritative state after installation. No summary becomes a new run, assignment, dispatch key, allowance, or completion event.
- After process death, recover candidate/attempt status conservatively. No auto-inference or tool execution before explicit eligible Resume; unknown remote summary outcome may have cost but cannot justify repeating a mutation. Reconcile effects through ordinary trusted read-back.
- Model switches invalidate any compatibility assumption about opaque reasoning/compaction items. Use the selected adapter's documented transformation or visible block; a readable summary is not a substitute encrypted token.

A possible layering, **not a new port implementation**: domain owns existing identities/gates and pure validity rules; application owns capture/admission/install and current-state projection; infrastructure renders provider input and parses native items privately; ACP/TUI only expose status and explicit human controls. Keep any future deterministic probes on the ordinary public application/provider/persistence seams.

### Later, only if evidence warrants it

- Provider-native compaction after exact subscription/OpenRouter route capability and replay are established. Keep native opaque items provider-private and retain audit provenance; do not require them for every provider. OpenAI explicitly says to replay the **entire canonical standalone compact output as-is**, which may include retained items; do not extract only its encrypted item or arbitrarily splice/prune that window. [N1]
- Task-local hierarchy/folding after simple projection/masking/checkpoint baselines. Existing child assignments, captured joins and durable results remain authoritative; a folding tree is only working memory, not a second scheduler.
- Speculative compaction after reliable stale-result rejection, Stop and late-usage handling; no idle inference behind human gates.
- Learned summary selection, ACE-style playbooks or RLM only under separately approved experiments. Training data, extra recursive calls, mutable memory policy and execution environment are real scope, not a free upgrade.

## 6. Matched, reproducible future evaluation plan

**Not executed and not authorized by this report.** Separate deterministic lifecycle proof from real-model quality/capability experiments. Approval of this research, a task manifest or a provider connection is not consent for paid calls, external mutations or merges.

### Freeze before running

Retain outside Harness main a versioned manifest containing: Harness candidate/base SHAs; fixture repository revision and starting artifacts; approved scope/checker revision; exact public commands/inputs; provider endpoint/auth class and catalog snapshot; requested/resolved model/effort and required parameters; adapter/config/prompt/instruction revisions; context window and trigger policy; seed where supported; route restrictions; retry policy; source tokenization/byte units; pricing date; probe source revision and expected oracle results. Never record credentials/private reasoning. Use disposable, approved resources with an independently maintained external effect/output ledger.

Do not add permanent tests/dependencies or debug APIs. Follow CODING_STANDARDS.md's applicable build/check commands for any future code. Deterministic fixtures may control model/clock/HTTP responses through public closure-backed seams; label them **controlled**, not proof of live accuracy or provider rejection. Actually kill/restart a process for crash durability; orderly cancellation is not equivalent.

### Comparison arms

| Arm | Behavior | Matching rule |
| --- | --- | --- |
| B0: current/full-history | Baseline adapter, no compaction; report oversized-context/record-limit failures honestly | Same task, fixed model/effort/instructions/tools and start state; do not silently omit failures. |
| B1: selective builder | Trusted current-state projection, bounded source-linked observations, on-demand reads; no summary inference | The required authority/evidence is identical; track retrieval calls and any extra tokens. |
| B2: deterministic masking | B1 plus older-observation masking, protected complete recent exchanges | Freeze window, excerpt and retrieval policy before test; preserve originals. |
| C1: structured checkpoint | B1 plus selected-model summary and retained tail | Freeze prompt/schema/cut points/trigger; count summary input/output/retries and changed cache behavior. |
| C2: hybrid | B2 plus occasional C1 | Isolate whether improvement comes from masking, summary, or extra inference. |
| Later: native/folding | Exact admitted route or task-folding integration | Separate compatibility qualification; no native result transferred across auth/protocol/model. Different architecture gets a separate table. |

Run paired task/seed cells, not a hand-picked successful trajectory. Preregister at least three seed/repetition cells per selected live-model configuration where feasible and enumerate all exclusions/timeouts/access failures. Determine a defensible task count/power from a separately authorized pilot, then freeze it before confirmatory runs; do not call a tiny convenience sample statistically conclusive. Include short tasks that should not compact and long tasks with one, several and **at least ten** reductions. Include a tail much larger than a single summary window and one giant tool result.

Quality comparison has two complementary views: (a) fixed model/effort/task and **reported actual total resources**, with no claim of equal spend; (b) an explicitly bounded *offline experiment* resource match to examine efficiency, disclosing censoring. An experiment's stopping rule is not a new production whole-spec cap or Decider deadline. Match active-context limits where the APIs permit, otherwise state the mismatch. Do not compare trained FoldAgent/extra recursive calls against one unaugmented call and attribute the gain to formatting.

### Required safety/failure cases and independent oracles

| Case | Controlled event / fault | Required observation, independent of summary prose |
| --- | --- | --- |
| Repeated compaction / long tools | Nested and parallel calls, giant output, summary-of-summary, discarded/failed summary | Valid call/result pairing; raw artifact hash retained; no phantom success; correct context-exhausted result if required data cannot fit. |
| Original evidence retrieval | Early disconfirming evidence needed after 1/3/10 reductions; distractors and a misleading checkpoint | Retrieve correct source/revision and use it correctly in an externally scored decision, not merely quote a summary. Record whether retrieval was attempted. |
| Opaque/private continuation | Reasoning, native compaction, model compatibility change | No exposed/synthesized private reasoning; exact adapter replay or visible block; no fallback model. |
| Concurrent child completion | Several children finish during summary, duplicate result delivery, parent busy, any/all joins | Durable captured membership/outcomes and exactly one consumption transition; no inference polling or false settlement. |
| Steering / input / approval | New higher-priority instruction during summary; transition into human wait | Stale candidate cannot launch superseded work; input ends run; answer starts later run on same identity; no inference behind waits. |
| Stop / subtree | Stop during send/stream/install; actual descendants versus siblings | No new admitted call/tool after Stop boundary; correct owned subtree stops, siblings untouched; completed/uncertain effects and late usage survive. |
| Unknown mutation | Kill after external effect but before local acknowledgement; summary claims failure/success | Independent external ledger/read-back; outcome remains Unknown until reconciled; zero blind duplicate mutation, no summary-authorized replay. |
| Restart / Resume | Real process kill during summary and while waiting for children | Durable identity/intent/usage and candidate disposition recover; zero inference/tools before explicit eligible Resume. |
| Provider/model change | Disable selected model, alias changes, quota/backoff, auth ownership changes | Captured/resolved identities and ordinary gates honored; no default model, reset allowance, guessed quota time or gate-bypassing timer. |
| Accounting | Failed/retried/cancelled summary, incomplete usage and late report | Each sent attempt counted once; wait not a call; estimates/unknowns separate; no reset/double count after checkpoint/restart. |
| Early final / unmerged readiness | Model says DONE, ticket closed, draft/ready PR, stale review or unmerged candidate | Independent revision/check/approval/host oracle: no complete spec until verified authorized merge; no inference of merge authority. Use simulated host for policy proof; real merge requires separate consent. |
| Corruption / installation race | Missing artifact, bad schema, conflicting revision, stale instruction snapshot | Conservative failure preserving evidence; no authoritative state reconstructed from summary. |

### Metrics, acceptance and retained evidence

Report **per attempted task and paired cell**, with all denominators: independent task success; later-evidence usefulness and source accuracy; missing/incorrect constraints; context-exhaustion rate; tool-pair validity; repeated-summary drift; false-completion, unauthorized-call, duplicate-mutation and lost-intent rates. Measure both recoverability and whether the model actually recovers needed evidence. Include adversarial artifact text trying to claim approval or redirect policy.

Report sent calls by purpose (worker, summary, verification, retrieval, routing if applicable), reported input/output/reasoning/cache-read/cache-write tokens when supplied, bytes and estimated tokens distinctly, actual/estimated/unknown monetary cost, end-to-end and compaction latency (median and tails), retries, failure/cancellation, cache rebuilds and total completion resources. Report cost per attempted task **and** cost per independently successful task; a cheap failed run is not a win. Subscription dollars remain unknown absent a valid basis. Preserve per-attempt records so late usage can reconcile without double counting.

Use paired differences and uncertainty intervals for quality/cost; stratify by length, number of reductions, provider/auth/model and failure case. Do not pool incompatible scores or convert benchmark totals into per-task prices without their denominators. Predeclare acceptable task-quality regression and the human adoption decision before confirmatory runs. For safety, require **zero observed** unauthorized inference/effects, false settlement, duplicate mutation and authority loss in the enumerated controlled cases; report denominators and coverage, since zero observations do not prove a universal zero rate. Any such failure blocks adoption. Native capability gaps block only the relevant option, never justify rerouting.

Retain manifest, probe sources, exact commands/expected/actual outputs, sanitized HTTP item types/request IDs, event/state snapshots, artifact hashes, cost provenance, independent checker results and per-cell exclusions outside disposable worktrees. Tie results to the exact candidate; invalidate affected evidence after edits. No superiority or production-readiness claim until the matched evidence exists and normal review/approval gates pass.

## 7. Source register

Source links below are commit-pinned where source is public. Documentation pages are mutable; retained snapshots and the external SHA-256 inventory fix what was inspected on the retrieval date. ArXiv links above pin paper versions; retained HTML/PDF and extracted text are in the evidence directory. Line-range validation checks existence/bounds, not semantic correctness or successful execution.

### H1

[H1]: #h1

Harness actual worker request.

- [Primary source](https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/infrastructure/src/create_model_step/openai.rs#L25-L87); snapshot `/Users/reilley/.t3/worktrees/harness/t3-c71d134a/infrastructure/src/create_model_step/openai.rs`.

### H2

[H2]: #h2

Harness private continuation.

- [Primary source](https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/infrastructure/src/create_model_step/context.rs#L1-L113); snapshot `/Users/reilley/.t3/worktrees/harness/t3-c71d134a/infrastructure/src/create_model_step/context.rs`.

### H3

[H3]: #h3

Harness execution lifecycle and baseline limits.

- [Primary source](https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/application/src/use_cases/create_message.rs#L118-L396); snapshot `/Users/reilley/.t3/worktrees/harness/t3-c71d134a/application/src/use_cases/create_message.rs`.

### H4

[H4]: #h4

Harness persisted record bounds.

- [Primary source](https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/conversations.md#L108-L133); snapshot `/Users/reilley/.t3/worktrees/harness/t3-c71d134a/docs/conversations.md`.
- [Primary source](https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/infrastructure/src/update_conversation/mod.rs#L24-L52); snapshot `/Users/reilley/.t3/worktrees/harness/t3-c71d134a/infrastructure/src/update_conversation/mod.rs`.

### H5

[H5]: #h5

Harness reported usage parser.

- [Primary source](https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/infrastructure/src/create_model_step/usage.rs#L1-L19); snapshot `/Users/reilley/.t3/worktrees/harness/t3-c71d134a/infrastructure/src/create_model_step/usage.rs`.

### C1

[C1]: #c1

Codex context and budget scopes.

- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/session/context_window.rs#L45-L125); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/session/context_window.rs`.
- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact_token_budget.rs#L1-L69); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact_token_budget.rs`.

### C2

[C2]: #c2

Codex trigger/model-switch/implementation selection.

- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/session/turn.rs#L1548-L1785); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/session/turn.rs`.
- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/model-provider/src/capabilities.rs#L21-L55); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/model-provider/src/capabilities.rs`.
- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/tasks/compact.rs#L1-L76); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/tasks/compact.rs`.

### C3

[C3]: #c3

Codex remote v2 request and transport.

- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact_remote_v2_attempt.rs#L170-L273); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact_remote_v2_attempt.rs`.
- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact_remote_v2.rs#L291-L405); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact_remote_v2.rs`.

### C4

[C4]: #c4

Codex local summary and fallback.

- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact.rs#L215-L471); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact.rs`.
- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact_model_fallback.rs#L1-L76); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact_model_fallback.rs`.

### C5

[C5]: #c5

Codex replacement installation and retained observations.

- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact_remote_v2_installation.rs#L73-L264); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact_remote_v2_installation.rs`.
- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/compact_retained_tool_outputs.rs#L1-L20); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/compact_retained_tool_outputs.rs`.

### C6

[C6]: #c6

Codex asynchronous candidate ownership and stale-history checks.

- [Primary source](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/session/async_compaction.rs#L24-L205); snapshot `/tmp/harness-completion-sources/openai--codex/codex-rs/core/src/session/async_compaction.rs`.

### P1

[P1]: #p1

Pi documented triggers, recovery and model overrides.

- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/docs/compaction.md#L25-L138); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/docs/compaction.md`.
- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/docs/compaction.md#L280-L463); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/docs/compaction.md`.

### P2

[P2]: #p2

Pi checkpoint prompts, projection and summary usage.

- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/compaction/compaction.ts#L507-L628); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/compaction/compaction.ts`.
- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/compaction/compaction.ts#L859-L1104); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/compaction/compaction.ts`.

### P3

[P3]: #p3

Pi runtime capture, cancellation and auto-compaction recovery.

- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L2910-L3065); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/agent-session.ts`.
- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L3106-L3400); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/agent-session.ts`.

### P4

[P4]: #p4

Pi serialization and persisted edits/checkpoints.

- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/compaction/utils.ts#L90-L163); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/compaction/utils.ts`.
- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/session-manager.ts#L1272-L1307); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/session-manager.ts`.
- [Primary source](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/session-manager.ts#L1360-L1400); snapshot `/tmp/harness-completion-sources/earendil-works--pi/packages/coding-agent/src/core/session-manager.ts`.

### O1

[O1]: #o1

OpenCode settings and explicit native/background modes.

- [Primary source](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/services/site/src/docs/content/compaction.mdx#L25-L206); snapshot `/tmp/harness-completion-sources/anomalyco--opencode/services/site/src/docs/content/compaction.mdx`.
- [Primary source](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/schema/src/config/compaction.ts#L1-L23); snapshot `/tmp/harness-completion-sources/anomalyco--opencode/packages/schema/src/config/compaction.ts`.

### O2

[O2]: #o2

OpenCode summary policy and precomputed-candidate reuse.

- [Primary source](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/compaction.ts#L124-L250); snapshot `/tmp/harness-completion-sources/anomalyco--opencode/packages/core/src/session/compaction.ts`.
- [Primary source](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/compaction.ts#L410-L663); snapshot `/tmp/harness-completion-sources/anomalyco--opencode/packages/core/src/session/compaction.ts`.

### O3

[O3]: #o3

OpenCode validation, staged shrinking and actual-call accounting.

- [Primary source](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/compaction.ts#L664-L949); snapshot `/tmp/harness-completion-sources/anomalyco--opencode/packages/core/src/session/compaction.ts`.

### O4

[O4]: #o4

OpenCode standalone Responses compaction utility.

- [Primary source](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/ai/src/protocols/utils/responses-compaction.ts#L21-L185); snapshot `/tmp/harness-completion-sources/anomalyco--opencode/packages/ai/src/protocols/utils/responses-compaction.ts`.

### G1

[G1]: #g1

Gemini standard-client compression policy.

- [Primary source](https://github.com/google-gemini/gemini-cli/blob/62bab5b232c15b925d31e655ce844ab939c882b7/packages/core/src/context/chatCompressionService.ts#L42-L103); snapshot `/tmp/harness-completion-sources/google-gemini--gemini-cli/packages/core/src/context/chatCompressionService.ts`.
- [Primary source](https://github.com/google-gemini/gemini-cli/blob/62bab5b232c15b925d31e655ce844ab939c882b7/packages/core/src/context/chatCompressionService.ts#L155-L200); snapshot `/tmp/harness-completion-sources/google-gemini--gemini-cli/packages/core/src/context/chatCompressionService.ts`.
- [Primary source](https://github.com/google-gemini/gemini-cli/blob/62bab5b232c15b925d31e655ce844ab939c882b7/packages/core/src/context/chatCompressionService.ts#L552-L802); snapshot `/tmp/harness-completion-sources/google-gemini--gemini-cli/packages/core/src/context/chatCompressionService.ts`.

### G2

[G2]: #g2

Gemini standard-client invocation and history update.

- [Primary source](https://github.com/google-gemini/gemini-cli/blob/62bab5b232c15b925d31e655ce844ab939c882b7/packages/core/src/core/client.ts#L1260-L1345); snapshot `/tmp/harness-completion-sources/google-gemini--gemini-cli/packages/core/src/core/client.ts`.

### G3

[G3]: #g3

Gemini separate compression service adjacency/fixed-model paths.

- [Primary source](https://github.com/google-gemini/gemini-cli/blob/62bab5b232c15b925d31e655ce844ab939c882b7/packages/core/src/context/contextCompressionService.ts#L285-L328); snapshot `/tmp/harness-completion-sources/google-gemini--gemini-cli/packages/core/src/context/contextCompressionService.ts`.
- [Primary source](https://github.com/google-gemini/gemini-cli/blob/62bab5b232c15b925d31e655ce844ab939c882b7/packages/core/src/context/contextCompressionService.ts#L460-L521); snapshot `/tmp/harness-completion-sources/google-gemini--gemini-cli/packages/core/src/context/contextCompressionService.ts`.

### A1

[A1]: #a1

Claude Code how it works.

- [Primary source](https://code.claude.com/docs/en/how-claude-code-works.md); snapshot `/tmp/harness-compaction-sources/claude-how.md`.

When context fills up, retained text lines 115–150.

### A2

[A2]: #a2

Claude Code survival/reloading semantics.

- [Primary source](https://code.claude.com/docs/en/context-window.md); snapshot `/tmp/harness-compaction-sources/claude-context.md`.

What survives compaction, lines 20–60. Closed-client documentation, not runtime inspection.

### A3

[A3]: #a3

Claude Code model/window controls.

- [Primary source](https://code.claude.com/docs/en/model-config.md); snapshot `/tmp/harness-compaction-sources/claude-model.md`.

Context window and auto-compaction, lines 819–879; per-model persistence qualification v2.1.288.

### A4

[A4]: #a4

Claude Code prompt cache and compaction cost.

- [Primary source](https://code.claude.com/docs/en/prompt-caching.md); snapshot `/tmp/harness-compaction-sources/claude-caching.md`.

Compacting the conversation, lines 175–190; also retained claude-costs.md and claude-troubleshoot.md, thrashing section.

### A5

[A5]: #a5

Claude Code public changelog.

- [Primary source](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md); snapshot `/tmp/harness-compaction-sources/claude-releases.md`.

Lines 169, 275–279 and 315: reported bug fixes, not measured safety rates. This changelog pin is separate from mutable docs retrieval.

### U1

[U1]: #u1

Cursor documented context mechanisms.

- [Primary source](https://cursor.com/docs/agent/overview.md); snapshot `/tmp/harness-compaction-sources/cursor-overview.md`.

Side chats section; also https://cursor.com/help/customization/context.md retained as cursor-help-context.md. Does not establish internal compaction policy.

### N1

[N1]: #n1

OpenAI compaction guide.

- [Primary source](https://developers.openai.com/api/docs/guides/compaction); snapshot `/tmp/harness-compaction-sources/openai-compaction.html`.

Retained text: server-side section lines 799–845; standalone lines 1227–1251. Entire compacted output is canonical; window must still fit.

### N2

[N2]: #n2

OpenAI standalone compact API reference.

- [Primary source](https://developers.openai.com/api/reference/resources/responses/methods/compact); snapshot `/tmp/harness-compaction-sources/openai-compact-reference.html`.

Compact conversation and returned object/usage; large retained HTML plus extracted text. General API contract, not SIWC entitlement.

### N3

[N3]: #n3

OpenAI Sign in with ChatGPT models/inference.

- [Primary source](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference); snapshot `/tmp/harness-compaction-sources/openai-siwc.html`.

Retained text lines 767–855; token-sharing reference retained separately as openai-siwc-reference.html. Navigation links to generic compaction do not establish auth-specific support.

### R1

[R1]: #r1

OpenRouter Responses overview.

- [Primary source](https://openrouter.ai/docs/api/reference/responses/overview.md); snapshot `/tmp/harness-compaction-sources/openrouter-responses.md`.

Stateless-only warning; explicit HTTP 400 for stateful request parameters.

### R2

[R2]: #r2

OpenRouter published OpenAPI.

- [Primary source](https://openrouter.ai/openapi.json); snapshot `/tmp/harness-compaction-sources/openrouter-openapi.json`.

JSON pointers: /paths/~1responses/post/requestBody; /components/schemas/ResponsesRequest, Inputs, CompactionTriggerItem, CompactionItem, OutputCompactionItem, MessagesRequest, AnthropicCompactionBlock, AnthropicUsageIteration. External openrouter-compaction-schema-extract.json retains the linked subset and source hash. A schema is not a live route test.

### R3

[R3]: #r3

OpenRouter middle-out message transforms.

- [Primary source](https://openrouter.ai/docs/guides/features/message-transforms.md); snapshot `/tmp/harness-compaction-sources/openrouter-transforms.md`.

Lossy middleware behavior, not native encrypted compaction.

### R4

[R4]: #r4

OpenRouter ScaleDown cookbook.

- [Primary source](https://openrouter.ai/docs/cookbook/building-agents/compact-agent-sessions-with-scaledown.md); snapshot `/tmp/harness-compaction-sources/openrouter-summarize.md`.

Separate compression pipeline/cookbook, not proof of native compaction on the selected route.


## 8. Limits and open human choices

- No live capability, model-quality, billing or safety experiment was performed. No vendor runtime tests were run. Public source pins are snapshots, not guarantees about installed releases or deployed accounts.
- Claude Code internals and Cursor compaction implementation remain unavailable in the inspected primary material. Native protocol coverage is uneven; OpenRouter's explicit schema is positive evidence but not route-level validation. Subscription compaction is **unverified**, not declared absent.
- Recent papers often change rapidly, rely on proprietary models and incompletely released artifacts, and optimize benchmark outcomes rather than factory safety. This report is not an exhaustive bibliography or independent replication.
- Human choices remain: whether any optional compression belongs in MVP; acceptable quality regression; whether/when to authorize provider-specific probes and paid paired trials; whether later hierarchy/training/recursive inference justifies scope. Existing durable authority, provider policy and merge gates are not up for automatic relaxation.
- This research-branch report may be committed/pushed for retention. That is not a submission for production integration, a PR, a merge request, tracker completion or cleanup authorization.
