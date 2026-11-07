# Spec completion and continuation controls: public prior art

Research for [HARNESS-46](https://app.plane.so/reilley/browse/HARNESS-46/), retrieved 2026-11-07 UTC. Research only; no requirement amendment, product implementation, tracker transition, paid inference, authentication experiment, PR, or merge. **“Goal” below is an upstream identifier, not a proposed Harness feature name.** Naming and adoption remain undecided.

## Findings in brief

1. **A genuine LLM-callable tool family exists in Codex:** `get_goal`, `create_goal`, `update_goal`, backed by persisted thread state and a runtime that starts another turn while the state remains active. This is not `update_plan`. At the inspected commit, `goals` is Stable and default-enabled, and the app-server installs the first-party extension. This proves that implementation/configuration, not universal availability in every client or older release. [C1–C4]
2. **Two other genuine first-party objective-continuation mechanisms exist:** Claude Code's native `/goal` (documented and release-noted) and OpenCode's built-in `opencode.goal` plugin, also invoked by `/goal`. These are **user commands, not verified LLM-callable goal tools**. Only Codex's family was verified as literal model tools; do not invent a second tool schema to meet a count. Claude's proprietary runtime is documentation/release evidence, whereas OpenCode's controller is public code. [A1, A2, O1, O2]
3. **Continuation is stronger than prompting, but completion remains weaker than Harness settlement.** Codex accepts a model's `complete` status without an external acceptance check. Claude and OpenCode use model evaluation of conversation evidence; a second model is not an independent repository/merge oracle. A deterministic Claude `TaskCompleted` hook can reject a task-completion mutation using a real check, but that example is not a product-wide verified-spec guarantee. [C4, A1, A3, O3–O5]
4. **The strongest reusable seam is before runtime settlement**, after ordinary tool/queue/recovery processing, with one admitted continuation rather than recursive prompts. Pi now names this distinction explicitly: `agent_end` is not `agent_settled`; `agent_before_settle` can append context and request one continuation. Codex's idle admission and OpenCode's idle hook implement related seams. [P1–P3, C5, O6]
5. **No inspected mechanism supplies the whole Harness contract.** In particular, model verdicts, a completion promise, “no background tasks,” and an emitted final answer do not establish approved-version criteria, captured joins, review/check provenance, human merge authority, and verified target merge. OpenCode's goal boot guard is useful but its general restart recovery can still automatically resume orphaned executions—directly contrary to Harness's explicit-Resume policy. [C4, C9, A1, O7, R1]

## Scope, versions, and evidence quality

The governing Harness input was the supplied immutable local snapshot `/tmp/harness-mvp-implementation/approved-spec.html`, SHA-256 `a70f27d50c2b69eb87e3c1b28f61ac1138ef4400e0f1d1a3ec033d65b86eb6bc`; canonical mutable specification: [HARNESS-19](https://app.plane.so/reilley/browse/HARNESS-19/). Its durable coordination, authority/Stop/recovery, retries/accounting, review/Retro/merge sections and V04/V08–V15 were read. This report evaluates prior art against those requirements; it does not replace them.

| Owner / evidence | Pin | What was actually inspected |
| --- | --- | --- |
| OpenAI, `openai/codex` | `d4a341bd52cc47ff635761334bde8a972d6de943`, committed 2026-11-07 04:06:26Z | Tool schemas/executors, first-party installation, state SQL, lifecycle/accounting, admission, child delivery/waits, suspension contract, source tests |
| Anthropic, `anthropics/claude-code` | `0ed94698aa0aeee471465fc9c867c2e8434c4d68`, committed 2026-11-06 17:03:45Z; changelog through **2.1.319** | Release history plus official live `/goal` and hooks documentation; **not runtime source** |
| Anomaly, `anomalyco/opencode` | `8bad789e1181aedb57327f41a2e24c4d710763d5`, committed 2026-11-07 03:40:10Z | Built-in registration, command/controller/schema, reviewer, idle admission, storage, child notifications, general crash recovery |
| Pi, `earendil-works/pi` | `085c84e98e7f4c950f2be606466fff4f75d8fe7c`, committed 2026-11-06 23:59:42Z; package **1.5.0** | Official installed README, SDK/extensions docs and relevant examples read first; pinned public docs/session implementation and todo/plan/subagent examples then cross-checked |
| Anthropic, `anthropics/claude-plugins-official` | `0412be76f50ab4f4f8abff3753efa9269fbbca5b`, committed 2026-11-06 23:51:04Z | Official **optional Ralph Loop plugin**, not native `/goal` implementation |
| Cursor | Official `https://cursor.com/docs/hooks`, retrieved 2026-11-07 | Public contract only; no runtime source or tested application version |

Pi's locally installed 1.5.0 is **not assumed byte-identical to public HEAD**, also labelled 1.5.0. SDK docs and todo example matched; extensions docs, plan and subagent examples differed. The continuation lifecycle documentation matched in the relevant sections; claims about the public plan fallback below are explicitly HEAD-specific. The older `badlogic/pi-mono` API resolved to the same public commit as `earendil-works/pi`; these are not two independent implementations.

Evidence coverage: **47 distinct cited primary artifacts**: **44 files in five first-party public repositories**, plus **three official documentation pages** (Claude goal/hooks and Cursor hooks), with 61 immutable code-line links. Deep implementation coverage is Codex, OpenCode and Pi; Claude core/Cursor are documented-contract coverage; Ralph is optional-plugin source coverage. Additional repository-tree screening is excluded from this count.

All positive findings below are **source-proven** or **officially documented**, not freshly executed runtime acceptance evidence. Public source tests were read, not run. Public GitHub trees were also screened for `block/goose`, `cline/cline`, and `google-gemini/gemini-cli`; that screening is **not evidence of absence** and no native goal-tool claim is made for them. Third-party packages have not been reclassified as native.

Mutable documentation response SHA-256s (raw HTTP bodies; fetched without service login):

- Claude `/goal`: `43608c20e08745e074c0d02cc85fb01e03fd3fc62b97ea49b5914c3f60f07813`.
- Claude hooks: `bbb34af5fcd844ae74bddb7c239662242882726d26c18aa4f0addddca2314bd6`.
- Cursor hooks: `afaed94da78bdc73bf19cd4239cd159ec4ce76c611ba64bd8df0cc9856b21b94`.

Temporary downloaded evidence is under `/tmp/harness-completion-sources/`. HTTP `.md` requests to Claude returned 403; the corresponding public HTML pages loaded successfully. Cursor's old `/docs/agent/hooks` redirected to `/docs/hooks`. No credential workaround or agent execution was used.

## Compact comparison

| Mechanism | Set/read/update and continuation | Who decides “done”? | Durable/wait/Stop boundary | Important limit |
| --- | --- | --- | --- | --- |
| **Codex native first-party extension** | Three actual model tools; host `thread/goal/*`; active state leads idle controller to `start_turn_if_idle` | Model `update_goal({status:"complete"})`; handler validates status/storage, not acceptance facts | SQL obligation/usage; serialized admission; separate paused/blocked/usage-limited states | No reviewed-revision/merge validator in completion handler; child delivery is best-effort and queue-only in inspected v2 path [C1–C9] |
| **Claude Code native `/goal`** | User condition becomes session-scoped prompt Stop hook; fresh evaluator can trigger next turn | Small fast model reads transcript; met/impossible clears condition | Active condition restored on resume; permissions unchanged; background results wake new turns | Evaluator cannot inspect files/execute tools independently; interactive timed check-ins violate Harness's no-model-polling wait rule [A1–A3] |
| **OpenCode built-in `/goal`** | Built-in plugin, user command, RPC get/changed; idle review then synthetic continuation | Session-context generation returns parsed verdict | Database KV state; own children/images defer review; old boot pauses goal; interrupt/failure pauses | No storage CAS; 50-round/3-stall/30-minute rules; general orphan execution auto-recovery bypasses a goal-only restart pause [O1–O9] |
| **Pi core lifecycle + optional extensions** | `agent_before_settle` / `turn_end` request continuation; queue APIs can trigger runs | Extension/host policy; no native goal criterion established here | Session entries persist; queues/closure flags are not durable coordination; abort suppresses boundary continuation | Not transactional boundary persistence; an unconditional continuation can loop; todo/plan markers are not acceptance [P1–P7] |
| **Claude official Ralph Loop plugin** | `/ralph-loop` writes local markdown; Stop hook blocks and replays original prompt | Assistant's exact promise text, or iteration bound | Session-id guard plus on-disk state; cancel removes file | Not native goal controller; no external verification, captured joins, approval-aware waits, or crash-safe execution ledger [R1–R3] |
| **Cursor native hook API + user scripts** | `stop` returns `followup_message`, submitted as next user message | Script/prompt policy, not platform acceptance | Generation/status/loop counters exposed; per-script limit defaults to 5 | Closed implementation; queued follow-up crash durability and Stop races not established [U1–U3] |

## 1. Codex: actual tools and hidden continuation controller

### Public surface and state transitions

Exact Responses API schemas (snake_case model arguments, `additionalProperties:false`, `strict:false`): [C1]

```text
get_goal({})
create_goal({ objective: string, token_budget?: integer })
update_goal({ status: "complete" | "blocked" | "paused" })
```

`objective` is required for create; `status` for update. Create is instructed to require explicit user or system/developer direction, not infer an obligation from an ordinary task. Its handler trims/validates the objective, validates positive/configured-maximum budgets, inserts an **active** record, and rejects creation while an unfinished record exists (SQL permits replacing a complete record). That authorization text is model guidance; the handler does not prove a trusted user approval record. Get returns the current record. Update accepts only the three statuses above and directly updates it. Responses contain `goal`, `remainingTokens`, `completionBudgetReport` (camelCase serialization). [C1, C4, C7]

Host control is separate: `thread/goal/set`, `thread/goal/get`, `thread/goal/clear`. Set's schema requires `threadId`, permits `objective`, `status`, `tokenBudget`, and `origin:"user"|"automatic"`; its comment explicitly says missing provenance does not supply user authorization. The host service records validated user intent before changing state under the runtime mutation permit. Protocol statuses are `active`, `paused`, `blocked`, `usageLimited`, `budgetLimited`, `complete`; SQL uses snake_case limited statuses. Review subagents do not see these tools; execution requires persistent thread state. [C3, C10–C12]

**Completion authority is still the model.** `update_goal` does not invoke tests, inspect remaining children, verify reviewed commits, check scope approval, or read back a merge. Likewise “same blocking condition for three consecutive turns” and “pause only at user's explicit request” are descriptions, not evidence audits in `handle_update`. Do not equate the status enum's validity with the truth of its claim. [C1, C4]

### Early final answers, continuation, and guards

Turn lifecycle records items/tool outcomes/accounting. `on_turn_stop` handles repeated failure/empty-output stops, flushes usage, records lineage for later automatic turns, and finishes turn accounting; it does **not** call the objective complete just because a final answer occurred. `on_thread_idle` calls `continue_if_idle`. The latter checks feature/tool availability, a persisted deferral, live thread existence and latest **active** state while holding a per-thread goal-state semaphore, builds continuation steering context, and calls `start_turn_if_idle` with trigger `"goal"`. A newly admitted turn is marked before the same permit can be used for stopping it. [C3, C5]

Core idle admission independently rejects pending trigger input, automatic Plan-mode starts, server drain, a non-idle thread, and (when supplied) a mismatched previous-turn identity. It reserves under the active-turn mutex and rechecks pending input. **Nuance:** the goal path uses `start_turn_if_idle`, not the separate expected-previous-turn continuation API; the generic supersession check is not proof that the goal caller supplied that guard. Goal-specific safety relies on its own lock, current record/IDs and accounting. This is a concrete single-admission mechanism, not a recursive “please continue” tool. [C5, C6]

State/usage is in SQL (`thread_goals`: thread and goal IDs, objective/status, optional budget, token/time usage and timestamps). Snapshot replacement also inserts a persisted continuation deferral in the same database transaction; turn start clears the deferral. Runtime timers, active IDs, failure counters and semaphores are process-memory state. `restore_after_resume` restores an active accounting baseline but does **not itself start a turn**. Persistence therefore proves recoverable obligation state, not exact accounting of an unflushed request or a universal “nothing executes until human Resume” invariant. [C7, C8, C13]

### Children, human waits, Stop, limits

- Descendant token usage is attributed to root accounting, but accounting inheritance is not child settlement. The inspected v2 completion controller captures terminal outcomes and routes a result with `trigger_turn:false`, explicitly **best effort**, with queued/failed metrics. V2 `wait_agent` waits for mailbox/input activity or a bounded timeout; its schema here is only `{timeout_ms?:integer}`, not a captured any/all child set. Do not transfer semantics from older `wait` variants or a test snapshot into a stronger v2 guarantee. [C3, C9, C14]
- Synchronous `request_user_input` awaits a tool result in the current turn. The separate `request_user_input_async` emits an async-delivery final-answer item with structured questions and returns `{"accepted":true}`; that handler alone neither persists a Harness-style ended-run/later-run transition nor changes the goal status. These are different mechanisms, neither evidence that an active obligation inherently respects Harness's input gate. [C15, C16]
- `paused`/`blocked`/`usageLimited`/`complete` clear active goal accounting; **setting paused is not itself execution cancellation** in `apply_external_goal_set`. User interruption and goal status are separate controls. The separate root suspension/shutdown API fences runtime starts and interrupts descendants, but explicitly says outstanding waiters are best effort, success need not await descendant cleanup, and callers must finish tree shutdown/fence writers. It is not the Harness Stop contract by name alone. [C5, C17]
- Runtime turn errors set blocked, usage-limit errors set usage-limited. Three qualifying automatic empty turns, or three turns with executed default-namespace `exec` failures and no successful tool, can stop the active obligation. Successful tools/other activity reset relevant counters; these narrow checks do not establish a general no-progress detector. There is no inspected code enforcing the description's three-turn *semantic* blocked audit. [C3, C8]
- Token-budget accounting and budget-limited steering exist, including descendant accounting; `max_goal_token_budget` can supply a maximum. These are **upstream differences, explicitly not recommended for Harness**. No whole-spec budget/time cap should be imported. No claim is made here about all lower-level Codex retry limits. [C3, C4, C5]

## 2. Claude Code: native condition loop, not the Ralph plugin

### Native `/goal`, documented behavior and versions

This feature is real: release **2.1.139** added `/goal`; **2.1.143** fixed evaluation during background work and added the eight-block protection. Later release evidence includes **2.1.234** unrecoverable-error clearing/background check-ins, **2.1.246** capped idle check-ins, **2.1.269** error retry/pause handling, and **2.1.274** a compacted-session restore fix. These are pinned release claims, not access to the closed runtime. [A2]

User surface: `/goal <condition>` sets/replaces the one active condition (up to 4,000 characters) and immediately starts a turn; `/goal` reads condition, duration/evaluated turns/spend/latest reason; `/goal clear` removes it (aliases `stop`, `off`, `reset`, `none`, `cancel`; `/clear` also removes it). This is a **slash command**, with no published `create_goal`/`update_goal` model-tool schema inferred. It is a session-scoped prompt-based Stop hook, subject to trust and hook-policy restrictions. Permission mode remains unchanged, so authorized completion does not bypass tool approval prompts. [A1]

After a turn, a **fresh small fast model**, configurable through `ANTHROPIC_DEFAULT_HAIKU_MODEL`, evaluates condition plus conversation and returns met/not-yet/impossible with a reason. Not-yet starts another turn with that reason. Met records achieved and clears; impossible records failed and clears. The evaluator **cannot run tools/read files independently**. It sees only surfaced evidence; transcript assertions and stale/misattributed test output remain threats. This is stronger than the worker marking its own checkbox but not validated external settlement. [A1]

### Waits, errors, restart, and loop limits

- Running subagents/background shells **defer evaluation**; their result is delivered as a new turn. But after 30 minutes a check-in can ask the model to inspect output/fix stuck work. Check-ins back off to 1h then 2h. Interactive idle check-ins (2.1.236+) can autonomously start turns; 2.1.246+ permits at most three between human prompts. `CLAUDE_CODE_GOAL_CHECKIN_MINUTES=0` disables check-ins and automatic retries. Non-interactive check-ins are only delivered at a subsequent turn end. **This model-driven idle inspection must not be copied into Harness's event-only wait path.** [A1, A2]
- Active condition is restored on session resume, including picker since 2.1.239, while count/time/spend baseline resets. Achieved/cleared conditions are not restored. This documents retention and restoration, **not an explicit permission barrier before any post-restart work**. There is no public crash-transaction/replay proof for condition plus queued continuation. [A1]
- On errors requiring user repair, docs say the condition clears: managed-by-Claude auth failure, exhausted credits, unrepairable context overflow, unavailable model. Host-managed auth is an exception. Other errors leave it set; in interactive 2.1.269+ transient failures retry automatically, with a pause after three retries; rate/usage limits and hook-ended turns pause, and a separately selected usage-reset continuation can later resume. Scope of that documentation is important: do not infer the same retry scheduling for all non-interactive hosts. [A1]
- Normal `Stop` does **not fire on user interrupt**; API errors use `StopFailure`, whose output cannot demand continuation. Normal Stop input includes `stop_hook_active`, `last_assistant_message`, and (when registry reachable) `background_tasks` / `session_crons`. Empty arrays mean no registered work; omitted/unavailable arrays must not be treated as proof of settlement. Stop output `{decision:"block",reason:string}` or `hookSpecificOutput.additionalContext` continues. After eight consecutive hook continuations, the next block is overridden; a tool call resets the count. Thus repeated no-tool early finals eventually return control, **not verified completion**; `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` can alter the threshold. `/goal` docs describe retaining the condition after the no-progress warning until next human prompt. [A1, A3]

### Deterministic completion and deferred-input hooks

`TaskCompleted` runs when `TaskUpdate` marks complete, or a teammate finishes a turn with in-progress tasks. Exit **2** prevents that completion and feeds stderr back. The official example runs `npm test`: this is a genuine external-fact veto at the mutation boundary, but only the configured check and event, not an approved-spec merge gate. `continue:false` behaves differently for teammate-triggered events and TaskUpdate-triggered events (ignored for the latter); use the actual contract, not a generic “all hooks can veto everything” claim. [A4]

Headless `PreToolUse` also documents `permissionDecision:"defer"`: tool does not run, run returns `stop_reason:"tool_deferred"` with `deferred_tool_use`, and the pending call stays on disk. Host waits without inference, then `claude -p --resume` with the same permission host re-evaluates it; `allow` plus `updatedInput` can supply the answer. This is useful prior art for ending execution for human input. It is **not automatic goal completion**, and the retention sweep (30 days default), permission-mode restoration caveats, and version-dependent unavailable-tool behavior matter. In 2.1.319, an unavailable deferred tool is retained and refuses input; older releases could lose it. [A5, A2]

`SubagentStop` can block a child's finish and pass a reason back to that child. It is not a durable captured join or parent merge authorization. Public docs do not prove execution-ownership Stop propagation, durable notification deduplication, or scope-version invalidation for `/goal`; these remain unknown here. [A3]

## 3. OpenCode: public first-party controller, restart caveat

`GoalPlugin.Plugin` is included in the built-in plugin list, with ID **`opencode.goal`**. It registers a **user `/goal` command**, not a model tool. Set creates `{id,objective,status:"active",rounds:0,maxRounds:50,boot,updatedAt}` in plugin KV; pause writes paused; resume requires paused, keeps ID/objective but resets rounds to 1, removes prior stall/reason state, updates boot and submits synthetic continuation; clear removes state. A goal is session-local: children/forks start without one. RPC `opencode.goal` has `get({sessionID})->State|null` and `changed({sessionID,goal})`; status enum is active/paused/complete, not a distinct blocked enum. [O1–O3]

### Exact end-of-work path

1. Runner `advanceToStep` drains ordinary inbox/control work. Only after a model step, when the session would otherwise go idle with no promotable input, it calls **`hooks.trigger("session","experimental.idle",{sessionID,background})`**. After the hook it checks inbox again and continues if input appeared. [O6]
2. Goal idle hook reads current active state; an earlier process boot settles it paused. Its own running **subagent/image** jobs defer evaluation; only own shell commands are offered as possible waits, because descendants' results wake their own parents. A returned wait is invalid when no own shell exists. [O4]
3. Otherwise it calls `ctx.session.generate({sessionID,prompt})`; generation uses current session context/model and returns text without mutating the session. Reviewer prompt asks for exact JSON `{verdict:"done"|"continue"|"blocked"|"paused"|"wait",reason:string,next_step?:string}`, says assistant claims are not evidence and asks for transcript tool results. This is an **LLM judge**, not a deterministic file/merge validation. Invalid verdict gets one more generation attempt. [O4, O5, O10]
4. Done writes complete; blocked/paused write paused with reason. Continue increments rounds/stalls, writes new state, then admits a **synthetic message** with `{source:"goal",goalID}` and reviewer feedback. Therefore state update and synthetic admission are distinct operations, not demonstrated atomic durable enqueue. Context reminders restate the objective after compaction. [O4]

### Limits, stale decisions, and recovery

- Constants are 4,000 objective characters, **50 maximum rounds**, **3 consecutive stalls**, **30-minute background inactivity** pause. Stall means same normalized reviewer reason or no tool calls since kickoff/continuation (missing kickoff after compaction counts as progress). It is a heuristic, not a semantic progress proof; do not import it into Harness as a fuzzy duplicate/effect classifier. The iteration and inactivity backstops are upstream policies, **not proposed whole-spec caps**. [O4]
- One context-overflow compaction per goal/round is allowed before pausing on repeated overflow. Unusable verdict, non-interrupt hook error, execution failure or user interruption pauses with an honest reason. Background inactivity timer is reset by descendant step activity; it pauses, not model-polls. The timer/compaction maps live in memory. [O4]
- Writes publish changed events. Controller `replace` re-reads ID, `updatedAt`, status before writing, so a later pause/clear is normally respected. **Source explicitly says storage has no compare-and-set**: the read/write pair is not a transactional race fence. After asynchronous judge return it checks ID/active status, then operates on latest state; this is not a durable approved-criteria generation/version check. [O4]
- Goal state is database-backed via plugin-namespaced KV. Startup scans old-boot active records and pauses them; source explicitly assumes one server process per database (a second would pause the first's live goals). That protects the **goal loop**, not the entire execution engine. General `SessionExecution` intentionally preserves an in-flight claim on shutdown; `SessionRestart` can automatically resume orphaned root/child executions, with a durable per-turn default **10** recovery-attempt limit and an explicit at-least-once-effects warning. Its service is inert until called; comments specify managed-server boot invokes it. **Do not claim OpenCode globally requires human Resume before execution just because goal status becomes paused.** [O7–O9]
- Child completion admits synthetic parent input with notification ID and then marks background notification complete, including failure/cancel state. Recovery can restore observers and resume children. This is meaningful source-level delivery/recovery evidence, but not proof of Harness's fixed any/all join capture, human-input-ended runs, or stop-by-actual-execution-owner across later parent runs. [O7, O11]

No goal-specific approval/quota gate was found in this controller beyond deferring while runner work is pending and pausing on execution failure. Whether every provider/permission/quota path gives the desired externally visible state is **unverified**, not inferred from an idle callback.

## 4. Pi: reusable settlement seam, not a native goal system

The installed official README explicitly says Pi skips built-in subagents and plan mode and directs users to extensions/packages. The examined `todo`, plan-mode and subagent implementations are **official examples**, not default product guarantees. No Pi-native goal-tool schema is asserted. [P0, P5–P7]

### Runtime boundary and queue contract

- Low-level `agent_end` may precede retry/final-attempt recovery, compaction retry and queued work. **`agent_before_settle` is actionable; `agent_settled` is notification-only** and means Pi will not automatically continue *that run*. Work requested from settled handlers is deferred until every handler finishes (not re-entrant dispatch). An extension can request new work afterward, so settled is not an approved-spec closure invariant. [P1–P3]
- `turn_end` and `agent_before_settle` handlers return entry drafts plus `continue`. Each sees prior proposals and rebuilt context. Allowed drafts: custom, custom_message, context_edit, compaction. A valid continuation needs runnable context; session validates it, commits drafts, then continues. `continue:true` requests **one next provider request** per boundary invocation; existing tools/queued work can satisfy it without an extra request. `continue:false` cannot cancel natural work. [P1, P2]
- Aborted/error responses are hard exits. Abort during pre-settle still commits valid entries but suppresses requested continuation. `abort()` cancels retry/compaction/branch summary and the agent then awaits idle. Stale extension context objects after replacement/reload throw; these are useful in-process lifecycle guards, not a persisted human-instruction generation protocol. [P1–P4]
- `sendMessage(...,{triggerTurn:true,deliverAs:"followUp"})` and `sendUserMessage` can enqueue/start work; `nextTurn` stores context for a later user prompt without starting inference. `appendEntry` persists extension state outside model context and startup handlers can rebuild it. SDK session history is persistent by default (or explicitly in-memory); this **does not make queued callbacks, closure flags or a multi-record obligation/admission transaction durable**. Documentation explicitly says boundary persistence is **not transactional** and warns an unconditional continuation can run forever. [P1, P2, P8]
- Retry handling uses settings `maxRetries` and cancellable exponential backoff; overflow recovery has a one compact-and-retry path. These are per-operation recovery policies, not a spec-wide budget. Extension UI waits expose `ui_prompt_start/end` around a blocking dialog but do not by themselves end the run as Harness requires. [P1, P9]

### Direct counterexamples: task lists and markers

`todo({action:"list"|"add"|"toggle"|"clear",text?:string,id?:number})` reconstructs state from active-branch tool results and changes `done` when the model toggles an item. It registers no finish veto. Persistence of a checkbox is not continued pursuit or verified acceptance. [P5]

At pinned public HEAD, plan-mode scans `[DONE:n]` markers (including thinking blocks) and also marks **all items complete** if the final assistant message matches `allStepsComplete`, whose regex accepts variants of “all/every steps done/complete/finished.” The installed older example lacked that fallback. Neither version runs an independent check; the execution branch returns from `agent_end` even when incomplete instead of unconditionally forcing the next step. Its nextTurn completion card is not an acceptance controller. This is a concrete example of why completion markers/prose must not settle a Harness spec. [P6]

The optional subagent example uses separate Pi subprocesses, abort propagation and awaited/parallel results; its documented maximum 8 tasks/4 concurrent are **example-specific**, not evidence of durable captured joins or a slot policy Harness should adopt. Parent/child persistence and late result consumption across a crashed coordinator are not guaranteed by that example. [P7]

## 5. Claude official Ralph Loop plugin: instructive, deliberately weaker

Do not confuse this first-party optional plugin with native `/goal`. `/ralph-loop` setup stores project-scoped `.claude/ralph-loop.local.md`: iteration, maximum iterations, completion promise, session ID, timestamp and prompt. Its Stop hook skips state owned by another session; legacy records without session ID fall through. It validates numeric fields, stops/removes state at a positive iteration limit, reads recent assistant transcript text, and exits if promise text matches. Otherwise it temp-file/renames the next iteration and returns `{decision:"block",reason:<same original prompt>,systemMessage:<iteration>}`. Maximum 0 means no plugin iteration limit; the runtime's independent hook protections may still apply. [R1, R2]

**The promise is not external evidence.** The hook does not run acceptance commands or inspect merge state. In fact, the Perl extraction leaves untagged output unchanged when no `<promise>` match occurs, and the comparison is literal text: untagged output exactly equal to the configured promise can also satisfy the comparison. That is a code-derived counterexample, not a freshly run probe. Source reads the transcript, despite current core docs warning it may not yet contain the latest final message on all versions. Corrupt/missing/unparseable state/transcript often removes state and allows exit, not “verified complete.” [R1, A3]

`/cancel-ralph` removes the state file. An ordinary user interrupt skips normal Stop; the plugin contains no execution-subtree Stop barrier or complete recovery protocol. On-disk prompt/iteration plus atomic rename is not durable external effects, a captured child join, or explicit restart admission. No child/approval/quota-aware branch is implemented in this Stop script. [R1–R3, A3]

## 6. Cursor: small, observable native continuation hooks

Official API input to **`stop`** is `{status:"completed"|"aborted"|"error",loop_count:number}` plus common fields. Output is `{followup_message?:string}`; a nonempty string automatically submits the next user message. Default is **5 auto follow-ups per script**, configurable `loop_limit:number|null` (`null` removes the limit). Common input includes stable `conversation_id`, **`generation_id` changing each user message**, model fields, `cursor_version`, roots and nullable transcript path. These are useful observability keys, **not evidence that stale returned hooks are fenced**. [U1, U2]

`subagentStop` also exposes status, task, summary, message/tool counts, modified files, child transcript and loop count; its follow-up is explicitly consumed **only when status is completed**, and repeats the child's hook until no follow-up/limit. Parent `stop` docs list aborted/error input but do not equally specify whether follow-up output is ignored for those statuses. A client script should not infer that missing guard; it requires an observable probe against a pinned app version. [U1, U3]

Hook config supports timeout, `failClosed` (false default), command/prompt evaluation; prompt hooks use an LLM `{ok:boolean,reason?:string}`. A shell hook can inspect external facts, but the platform provides no spec criterion ledger. `sessionEnd` is fire-and-forget (including window/user close) and ignores output. `sessionStart` also does not enforce its nominal `continue:false` output. None establishes no-after-close execution, crash-safe queued follow-up, durable join settlement, or approval/usage-wait survival. Cloud docs additionally distinguish command hooks from unavailable prompt-hook support and early read-only phases where hooks do not run. [U1, U4]

## Fit to the approved Harness constraints

This is evaluation/recommendation, **not a new implementation spec**.

| Existing contract | Narrow reusable evidence | What Harness must continue to own |
| --- | --- | --- |
| Durable CoS per approved spec; criteria/version cannot be silently changed | Codex SQL objective/status; OpenCode persisted ID; Pi explicit context/state entries | Spec/approval identity and snapshots, agent vs assignment/run/generation, version-bound evidence. None of the foreign model-complete verdicts may close approved work. |
| Response completion is not settlement | Pi pre-settle/settled; Codex idle continuation; OpenCode idle inbox recheck | Reconcile approved outstanding work and required follow-up before deciding whether another action is eligible; preserve terminal reply separately from assignment/spec state. |
| Human input ends run; reply starts later run; children may keep running | Claude headless defer is the closest public ended-invocation example; OpenCode typed child notices | Durable run-ending input/approval state and surviving execution ownership. A blocking tool/UI dialog is not equivalent. Parent gates must not silently Stop its children. |
| Captured any/all waits; no model polling | OpenCode own-job deferral/event result; Pi queue timing | Fixed captured execution sets, durable outcome/notification consumption and safe parent wake. Do not copy Claude timed model check-ins or Codex timeout-return/model-wait loops as a polling coordinator. |
| Stop actual subtree; restart explicit Resume; nothing while closed | Pi abort suppresses continuation; Codex admission fence; OpenCode old-boot pause | Actual execution-owner graph across run changes, descendant acknowledgement/uncertain effects, shared-storage admission fence, restart/closed gate dominating every timer/notification. OpenCode generic auto-recovery is a counterexample, not a template. |
| Legitimate quota/approval/configuration waits | Codex separate usage-limited/blocked; Claude defer/permission/error distinctions | Actual provider scope/reset facts, bounded transient retries, no guessed slots/reset time, no inference while waiting; explicit input/approval/Stop/restart must dominate timers. |
| Repeated early final/no progress must be honest, without whole-spec caps | Codex narrow 3-empty/exec-failure guards; Pi continuation-loop warning | A visible blocked/no-progress outcome tied to concrete outstanding work/evidence, preserving approved retry/rework allowances. No global duration/spend cap, semantic duplicate/effect classifier or imported OpenCode 50-round cap. |
| Independent review, separate merge authority, spec complete only after verified merge | Claude TaskCompleted external check illustrates an evidence veto, not merge authority | Exact reviewed revisions/checks, assembly and Retro, settled required children/follow-up, human revision-specific merge approval or direct-human merge detection, host read-back/reconciliation. Readiness/closure/merge/cleanup remain distinct. |

## Narrow recommendation

**Provisionally prefer a backend-owned pre-settlement eligibility check plus one durable, event-driven continuation admission for already approved actionable work.** Use the shape of Pi's boundary distinction and Codex's serialized start-if-idle admission; use OpenCode's own-job wait discipline and honest pause reasons as examples. Do not adopt a general objective framework or name the Harness feature “goal.” This recommendation is subject to parent/user review before any requirements change.

A model's final reply/completion claim should be input to that check, not authority. When approved work remains actionable and current gates permit it, arrange a single next normal execution through the existing durable execution path. If a captured child/input/approval/provider/configuration/restart gate blocks progress, record the precise wait and **do not call a model merely to ask whether it is still waiting**. No automatic merge follows from continued execution. Before spec completion, use the approved merge-evidence contract, not a language-model verdict. These conclusions follow the contrasts above, particularly Codex's unvalidated complete mutation, Claude's transcript-only evaluator, OpenCode's non-CAS/restart gaps and Pi's nontransactional extension boundary. [C4–C9, A1, O4, O7, P1–P3]

Do not import foreign policy numbers wholesale: Codex token budgets, OpenCode max rounds/inactivity limits, Ralph iteration count, Cursor loop limit and Claude evaluator/time clauses are descriptions of upstream behavior. Harness already has bounded transient retries and ticket/final rework allowances; **no whole-spec time/spending caps, including opt-in**, new global/per-role slots, fuzzy duplicate/effect classifiers, periodic model polling, or deadline on pending native Decider work is recommended.

## Observable public-interface probes to propose, not run here

Use ordinary production command/query/event interfaces once implemented, with temporary external probes and controlled ordinary effect ports as the approved verification strata require. For third-party checks, pin the app/version and use a disposable repository only after any runtime/paid authorization. No debug controller endpoint or permanent tests are proposed.

| Case / approved verification family | Public observation / discriminating result |
| --- | --- |
| **Premature final, actionable work** (V04/V08) | Script a normal worker's plain final answer while an approved action remains. Transcript may finish; state must not become spec complete. Observe exactly one accepted next execution, not duplicate recursive prompts. |
| **False success signal** (V13/V14) | Return complete/“all steps done”/promise while checks fail or merge is absent. Inspect normal status/evidence: not settled or merged. Separately show an actual verified merge can complete even if old prose says otherwise. |
| **Captured join** (V08) | Capture A+B all/any; later spawn C; complete A, stop/fail/input B. Observe fixed set semantics, truthful partial outcomes and no model polling. Duplicate B notification cannot duplicate parent follow-up. |
| **Human question while child runs** (V08/V09) | Parent requests input, ending its run; child progresses and result is retained. No parent inference until answer; answer starts a later run on same agent with original child ownership intact. |
| **Approval or unknown-reset quota wait** (V09/V10) | Parent pauses; child result/timer arrives. Verify no automatic inference or authorization bypass; known reset and unknown reset remain different. Stop/close/restart supersedes a pending retry/reset. |
| **Stop subtree and race** (V09/V11) | Stop parent with background descendants and unrelated sibling/creator-only relation. Observe actual subtree cancellation, preserved effects/usage, no new continuation from a late final, and unchanged siblings. |
| **Crash between decision and enqueue** (V08/V11) | Kill after durable result/state, before continuation acknowledgement; restart/query without Resume. No model/tools. After explicit Resume, reconcile effects and admit only the still-eligible work once. Repeat with kill after remote merge before local acknowledgement. |
| **Stale decision/version** (V04/V09/V14) | Delay completion/continuation result; steer or amend/approve a new criteria version, Stop or close. Old result may be retained as evidence but cannot start stale actions, waive criteria or authorize merge. |
| **Repeated early final/no progress** (V10/V13) | Repeated text-only finals or the same concrete failure lead to visible blocked/no-progress handling, not spec complete, silent abandonment or endless spend. Existing allowances persist through rename/routing/restart. Product policy must be reviewed; this report does not introduce a new whole-spec limit. |
| **Merge boundary** (V14/V15) | Ready PR without merge authorization: no merge. Authorized exact candidate plus host refusal/unknown result: no false completion. Direct human merge with read-back: complete; cleanup blockers separately visible. |
| **Upstream diagnostic probes** | Codex: model complete while a child remains; inspect v2 parent notice vs turn trigger. Claude: interrupt before Stop, deferred input restore, background completion vs timed check-in. Pi: abort inside pre-settle; two handlers requesting continuation; process kill between entry append and next request. Cursor: aborted/error parent stop returning follow-up, delayed old-generation response, loop-limit and close/reopen behavior. These distinguish documented gaps; no result is asserted. |

## Unknowns and open questions

- **Not a requirements question:** exact names/tools/UI and whether to adopt continuation remain for parent/user review. The research supports a narrow backend seam, not a final public schema or implementation plan.
- **Public evidence gap:** no second literal LLM-callable goal-tool family was verified; two additional real native command/controller mechanisms were verified instead. A broader third-party package audit could find others but is not needed to justify the seam and should not delay existing approved work.
- **Closed implementations:** Claude runtime transactions, full captured-join semantics, Stop subtree ownership, stale-generation guards and Cursor follow-up persistence/admission are unknown beyond the cited docs. Doc freshness is pinned by retrieval/hash, not an invented source commit.
- **Source inspection is not reliability measurement:** crash recovery, competing writes, tool/input races and no-progress policies need public-interface evidence before adoption. OpenCode explicitly lacks storage CAS and warns at-least-once recovery; Pi explicitly says boundary persistence is nontransactional. Codex persistence and admission do not prove Harness's stronger authority/merge contract.
- **No new product interview needed:** the approved Harness snapshot already resolves durable identity, human-input run boundaries, wait/Stop/Resume authority, no whole-spec caps, and verified merge. What remains is choosing a narrow mechanism and proving those contracts, not re-deciding them.

## Report verification

This is a documentation-only research change. `git diff --check` passed; cited repository paths were matched to downloaded immutable source and line ranges checked against file lengths. Source counts distinguish unique files/pages from repeated line links. Raw documentation and approved-spec hashes were computed locally. No product build/test/runtime probe was run; no machine-wide dependency installation was attempted. Source tests mentioned here remain read-only evidence, not executed test passes.

The retained copy is `/tmp/harness-mvp-implementation/research/spec-completion-continuation-controls.md`. The research branch is `research/spec-completion-continuation-controls`; its commit/push result is returned to the parent separately rather than encoded circularly in this file.

## Source ledger

Line links pin public code; named documentation sections are mutable and were retrieved/hash-pinned above. Reference groups such as C1–C4 mean the listed individual sources, not an assertion of evidence from an uninspected implementation.

### Codex — OpenAI

- **C1:** [Exact goal model-tool names, descriptions and schemas](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/spec.rs#L9-L94).
- **C2:** [Stable/default-enabled feature](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/features/src/lib.rs#L2203-L2208); [app-server installation/configuration](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/app-server/src/extensions.rs#L85-L100).
- **C3:** [Visibility, persistence requirement and lifecycle registration](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/extension.rs#L101-L208); [turn lifecycle, errors, descendant usage and tools](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/extension.rs#L242-L657).
- **C4:** [Goal executor: create/get/update and response serialization](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/tool.rs#L50-L308); [budget validation/response](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/tool.rs#L419-L493).
- **C5:** [External status effects](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/runtime.rs#L192-L277); [resume baseline and idle controller](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/runtime.rs#L412-L556).
- **C6:** [Core start-if-idle reservation and rejection conditions](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/session/turn_input.rs#L511-L655).
- **C7:** [SQL get, snapshot/deferral, insert/update](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/state/src/runtime/goals.rs#L43-L370).
- **C8:** [In-memory accounting and exact execution/empty-response guards](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/accounting.rs#L19-L252).
- **C9:** [Captured child terminal routing; queue-only/best effort](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/agent/control/completion.rs#L1-L155).
- **C10:** [Host set schema including provenance and status enum](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/app-server-protocol/schema/json/v2/ThreadGoalSetParams.json#L1-L67).
- **C11:** [Host set/get/clear paths and ordered response/runtime effects](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/app-server/src/request_processors/thread_goal_processor.rs#L60-L320).
- **C12:** [Service intent-before-state contract and mutation serialization](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/ext/goal/src/api.rs#L146-L266).
- **C13:** [Persisted record/status representation](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/state/src/model/thread_goal.rs#L1-L103).
- **C14:** [V2 wait_agent timeout/activity semantics and schema](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/tools/handlers/multi_agents_v2/wait.rs#L40-L192).
- **C15:** [Synchronous request_user_input waits for response](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/tools/handlers/request_user_input.rs#L59-L100).
- **C16:** [Separate asynchronous question tool](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/tools/handlers/request_user_input_async.rs#L22-L157).
- **C17:** [Root suspension/shutdown guarantees and limits](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/core/src/codex_thread.rs#L648-L662).
- **C18:** [Source test for three empty automatic continuations, read not executed](https://github.com/openai/codex/blob/d4a341bd52cc47ff635761334bde8a972d6de943/codex-rs/app-server/tests/suite/v2/thread_goal_empty_responses.rs#L30-L165).

### Claude Code — Anthropic native docs and releases

- **A1:** [Native /goal: full condition, evaluation, resume, background/errors and requirements contract](https://code.claude.com/docs/en/goal), particularly `#how-evaluation-works`, `#resume-with-an-active-goal`, `#background-work-defers-evaluation`, `#when-a-turn-fails`.
- **A2:** [2.1.139 addition](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md#L6476-L6482); [2.1.143 fixes](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md#L6336-L6345); [2.1.246 check-in bound](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md#L4290-L4296); [2.1.269 recovery](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md#L3487-L3495); [2.1.274 restore fix](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md#L3180-L3187); [2.1.319 deferred-call fix](https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md#L3-L40).
- **A3:** [Stop](https://code.claude.com/docs/en/hooks#stop), [Stop input/control](https://code.claude.com/docs/en/hooks#stop-input), [StopFailure](https://code.claude.com/docs/en/hooks#stopfailure), [SubagentStop](https://code.claude.com/docs/en/hooks#subagentstop).
- **A4:** [TaskCompleted mutation veto and official npm test example](https://code.claude.com/docs/en/hooks#taskcompleted).
- **A5:** [PreToolUse decision control, deferred tool and resume contract](https://code.claude.com/docs/en/hooks#pretooluse-decision-control).

### OpenCode — Anomaly

- **O1:** [Built-in GoalPlugin import/registration](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/internal.ts#L104-L105), [built-in list](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/internal.ts#L240-L260).
- **O2:** [User command set/pause/resume/clear](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/goal.ts#L343-L435).
- **O3:** [State and get/changed RPC schemas](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/schema/src/goal.ts#L1-L46).
- **O4:** [Goal controller: persistence guard, reviewer, idle loop and limits](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/goal.ts#L15-L228); [watchdog/events/boot recovery](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/goal.ts#L230-L341).
- **O5:** [Exact reviewer prompt/verdicts](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/goal/judge.txt#L1-L20).
- **O6:** [Runner idle hook and inbox recheck](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/runner/llm.ts#L64-L219).
- **O7:** [General restart recovery, attempts and at-least-once warning](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/execution/restart.ts#L16-L100); [child recovery and root automatic resumes](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/execution/restart.ts#L166-L265).
- **O8:** [Execution claim persistence and interruption semantics](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/execution.ts#L18-L183).
- **O9:** [Plugin namespaced KV](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/plugin/host.ts#L663-L687); [database KV implementation](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/kv.ts#L36-L82).
- **O10:** [Reviewer generation path and session model/context](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/generate.ts#L25-L68).
- **O11:** [Terminal child notification admission and completion](https://github.com/anomalyco/opencode/blob/8bad789e1181aedb57327f41a2e24c4d710763d5/packages/core/src/session/subagent-completion.ts#L21-L46).

### Pi — official runtime/docs/examples

- **P0:** [README: deliberately omitted default features](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/README.md#L22-L29).
- **P1:** [Lifecycle/boundary API, abort and nontransactional persistence caveats](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/docs/extensions.md#L663-L750).
- **P2:** [Runtime preview, draft commits and settled-event dispatch](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L1003-L1175); [post-run and pre-settle checks](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L2050-L2115).
- **P3:** [SDK lifecycle, persistent session manager, queue and abort semantics](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/docs/sdk.md#L25-L99).
- **P4:** [Abort implementation](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L2598-L2618); [stale extension contexts](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/extensions/runner.ts#L728-L743).
- **P5:** [Todo schema and state reconstruction/model mutation](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/examples/extensions/todo.ts#L30-L217).
- **P6:** [Public-HEAD plan completion behavior](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/examples/extensions/plan-mode/index.ts#L257-L294); [marker and prose recognition](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/examples/extensions/plan-mode/utils.ts#L155-L194).
- **P7:** [Optional subagent example, subprocesses, abort and limitations](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/examples/extensions/subagent/README.md#L1-L180).
- **P8:** [Message queue delivery, triggerTurn and appendEntry APIs](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/docs/extensions.md#L1627-L1735).
- **P9:** [Retry budget and abortable delay](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L3920-L3971); [overflow compact/retry](https://github.com/earendil-works/pi/blob/085c84e98e7f4c950f2be606466fff4f75d8fe7c/packages/coding-agent/src/core/agent-session.ts#L3090-L3203).

### Ralph Loop — official optional Anthropic plugin

- **R1:** [Entire Stop-hook controller](https://github.com/anthropics/claude-plugins-official/blob/0412be76f50ab4f4f8abff3753efa9269fbbca5b/plugins/ralph-loop/hooks/stop-hook.sh#L10-L191), particularly session guard L27–35, limit L60–65, promise L126–142, continuation L144–191.
- **R2:** [Setup state and counters](https://github.com/anthropics/claude-plugins-official/blob/0412be76f50ab4f4f8abff3753efa9269fbbca5b/plugins/ralph-loop/scripts/setup-ralph-loop.sh#L129-L158).
- **R3:** [Cancel command](https://github.com/anthropics/claude-plugins-official/blob/0412be76f50ab4f4f8abff3753efa9269fbbca5b/plugins/ralph-loop/commands/cancel-ralph.md#L1-L18); [plugin hook binding](https://github.com/anthropics/claude-plugins-official/blob/0412be76f50ab4f4f8abff3753efa9269fbbca5b/plugins/ralph-loop/hooks/hooks.json#L1-L15).

### Cursor — official public contract, not open runtime

- **U1:** [stop follow-up, status and loop limits](https://cursor.com/docs/hooks#stop).
- **U2:** [Common schema with generation ID](https://cursor.com/docs/hooks#common-schema).
- **U3:** [subagentStop completion-only follow-up](https://cursor.com/docs/hooks#subagentstop).
- **U4:** [Configuration, lifecycle, hook failures/types and cloud support](https://cursor.com/docs/hooks).
