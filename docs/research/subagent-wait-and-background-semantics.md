# Research: subagent waits and background execution

Verified: **2026-11-06 UTC**. Question: [HARNESS-12][Q]; context: [software-factory map HARNESS-1][M]. This is research and a set of alternatives, **not an approved product policy**.

## Findings in brief

1. **Foreground waiting, multiple concurrent children, and continuing parent work are compatible.** Claude Code explicitly supports foreground subagents and moving an already-running foreground task to the background; its SDK exposes the same operation. Codex separates spawning from waiting. Neither example requires a provider generation to remain open while a parent waits. [C1][C2][O2][O3]
2. **Ending parent inference is not the same as finishing the parent's work.** The current Claude Python SDK explicitly keeps the control channel alive after a result when child work or its completion processing remains outstanding. Pi distinguishes `agent_end` from `agent_settled`. T3 separately exposes `workState`, terminal-run information, and pending children. These are different boundaries, not interchangeable spellings of “done.” [C4][P2][P3][T1]
3. **A foreground-to-background transition should not mean cancelling and respawning the child.** Claude's documented transition releases the waiting tool call, continues the parent turn, and later delivers a task notification. Codex's spawn/wait split provides a different design: the child is independent of the parent's choice to wait. [C2][O2][O3]
4. **Pi is not a native subagent scheduler.** Its README explicitly omits built-in subagents. Its shipped example extension supports foreground single/parallel/chain delegation, but not detach, steering, or durable child recovery. The SDK supplies session/event/message primitives from which a host can build those features. **T3's delegation contract belongs to T3, not Pi.** [P1]–[P4][T1]
5. **Persistence is not automatic execution recovery.** Claude documents resumable child transcripts; Codex source exercises cold reconstruction and subsequent follow-up; Pi persists sessions. None of that establishes exactly-once external effects or an across-crash, acknowledged completion mailbox for Harness. T3's exposed schemas alone do not establish its crash guarantees. [C1][O6][P2][T1]

**Source-backed direction to consider:** separate child execution, parent waiting, parent readiness, and durable-run terminality. Offer a real foreground wait plus a non-destructive background action and explicit some/all joins. Then choose between a suspended logical run, a new-run-on-wakeup policy, or a bounded hybrid. Research does **not** select that policy; in particular, “every parent wait ends its durable run” remains unapproved. [Q][M]

## Evidence and version boundaries

Only primary documentation, first-party public source, installed Pi files, repository documents, and exposed T3 tool contracts were used. No child, workflow, model request, paid experiment, or secret request was launched. Findings are documentary/source verification, not runtime probes.

| Surface | Evidence baseline |
| --- | --- |
| OpenAI Codex | Latest-release API reported `rust-v0.167.0`, resolved to **`086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a`**. Code citations below pin that commit. Initial exploration used main `1a2f5babe83bc4b8d4d89701a01b7044a0ea6135`; both wait handlers, tool specifications, and completion controller were checked against the release and were identical. Both V1 and V2 exist; this report does not assert which feature/model/client selects either. Public docs are a dated, mutable snapshot. [O0]–[O6] |
| Claude Code | Public changelog at **`0ed94698aa0aeee471465fc9c867c2e8434c4d68`** starts with **2.1.319**. Public docs describe current behavior and identify several minimum versions. The closed CLI implementation was not inferred from its behavior. [C0][C1] |
| Claude Agent SDK | TypeScript changelog **0.3.319**, commit **`49dd8ef10dd5cb012dc9fe10703376489850c36c`**; Python source **0.2.179**, commit **`2209f2c134fee914c0f75733b631a5fc238ca846`**. Python control-channel observations are limited to that source, not claims about the closed CLI internals or every older SDK. [C3][C4] |
| Pi | Installed **`@earendil-works/pi-coding-agent` 1.5.0** at `/Users/reilley/.bun/install/global/node_modules/@earendil-works/pi-coding-agent`. Fully read `README.md`, `docs/sdk.md`, `docs/extensions.md`, and the subagent example's `README.md` and `index.ts`. All five files were byte-compared with tag `v1.5.0`, commit **`f9a26365568127461c47a07a37dcb42c140a3ac5`**, and matched. [P1]–[P4] |
| T3 integration | Live tool schemas and this session's orchestration instructions, observed on verification date. Installed desktop metadata says **0.1.7**, but that is **not** verification of the executing server's source commit/version. No T3 internals or crash behavior are claimed from the desktop version. Relevant schema excerpts are preserved below. [T1] |
| Harness | Checkout baseline **`6b4512d6ef7b0aa38783ff939bfd0f76234f41a4`**; `AGENTS.md`, `CODING_STANDARDS.md`, `GLOSSARY.md`, `docs/conversations.md`, issue-tracker instructions, full research question, and map read. No product code changed. [H1][Q][M] |

The Claude SDK docs now live under `code.claude.com/docs/en/agent-sdk/`. An older `platform.claude.com` URL redirected to an overview rather than the requested page; the specific pages cited here were fetched from their current owning location. Documentation is not a guarantee that every installed client has the latest behavior.

## Keep the lifetimes separate

These distinctions are analytical vocabulary for this report, not changes to Harness's glossary.

| Concept | Meaning here | Important non-equivalence |
| --- | --- | --- |
| Provider model invocation | One call to a model API, producing a generation or tool requests | A harness can wait on a tool after that generation has finished. An async function waiting is not itself continuous paid inference. |
| Model step | Harness's existing name for one generation | A tool-requesting step does not finish the Harness turn. [H1] |
| Turn | Must be qualified by owner | Harness: one user message plus model/tool steps answering it. Pi: one model response plus its tool calls. Claude's agent-loop guide also calls one loop pass a turn, while its SDK/control-channel lifecycle discusses larger main-thread turns ending in result frames. Preserve the actual event/API boundary rather than translating all occurrences to one enum. [H1][P3][C4][C5] |
| Conversation/session/thread | Addressable history that can receive later work | History can survive an execution attempt; an idle thread is not necessarily closed, cancelled, or complete forever. [H1][P2][C1][O5] |
| Task | Delegated objective/handle | May have several attempts/runs. Claude's current task events explicitly distinguish `task_id` from per-run `run_id`; T3 also distinguishes task, child thread, and child run. [C2][T1] |
| Durable run | Proposed Harness orchestration record grouping execution toward an outcome | Not automatically a provider request, SDK `prompt()` call, Claude result frame, or entire conversation. HARNESS-1 requires a later run after human input, but does **not** decide the same policy for child waits. [M][Q] |
| Parent work state | Whether work is runnable, waiting, or has a deliverable result | T3 exposes `working`, `waiting_for_children`, `result_available` independently of latest terminal-run fields and `hasPendingChildRuns`. Harness can use a similar projection without copying T3's implementation. [T1] |

There are also three different meanings of “blocking”:

- **Parent-model blocking:** the parent cannot take its next model step until a tool returns.
- **Runtime yielding:** an asynchronous wait releases execution to other tasks/children, without requiring another parent model call. Codex's `FuturesUnordered`, status subscriptions, and Tokio timeouts provide concrete source examples. [O3]
- **Durable suspension:** a persisted predicate/continuation can be recovered after process loss. An in-memory promise or channel alone does not establish this property.

Likewise, “wakeup” may mean resolving a currently awaited tool, queueing context for the next step, scheduling a new model turn, or creating a later durable run. Those are distinct choices. Codex V2 even exposes queue-only messages separately from follow-up tasks that trigger an idle target. [O4][O5]

## OpenAI Codex

### Spawn first; wait only when needed

The public docs describe parallel children, returning summaries, inspecting/switching agent threads, and asking Codex to steer or stop a running child. They show “wait for all” as a requested orchestration behavior. This is product-level documentation, **not evidence that each low-level wait call is an all-child join**. [O1]

At the pinned release, V1 `spawn_agent` awaits child creation/submission and returns its ID/nickname, not its final answer. Waiting is a separate tool. Thus multiple children can be started and the parent can do independent work before choosing a join point. “Foreground” can describe the parent currently awaiting their results; it need not be a child-process execution mode. The inspected Codex tool surface has no Claude-style `run_in_background` flag or documented Ctrl+B-equivalent child conversion; do not invent one. [O2][O3]

### Two wait protocols coexist

**V1, `multi_agent_v1.wait_agent(targets, timeout_ms)`:** [O3]

- Subscribes to each target's status and reads an initial snapshot.
- If any targets are already final, returns those final snapshots immediately.
- Otherwise waits concurrently for a target to become final; after the first final result, drains other immediately available final results.
- Returns a status map and `timed_out`; empty status means timeout. This is **some/any-final**, not wait-all. To wait for all, the coordinator must retain the target set and repeat for the remaining targets, handling failure/not-found as well as success.
- Timeout returns to the parent without invoking child cancellation. Waiting is an async tool execution associated with the current turn; the handler does not end a separate durable orchestration run.

**V2, `wait_agent(timeout_ms)`:** [O3][O4]

- Takes **no target set**; waits for mailbox activity or steered user input, with a timeout.
- Reports wait completion, interruption, or timeout, not the child-result content itself.
- Mailbox activity includes messages and final-status notifications, so “woke” does **not** imply a child finished, much less all children finished.
- A coordinator must classify received updates and evaluate its own join predicate.

Both are foreground waits from the parent model's perspective and yielding waits from the runtime's perspective. In neither inspected handler is “waiting” implemented by repeatedly asking the model whether a child has finished. [O3]

### Delivery, steering, and cancellation

V1's specification promises a final-status notification in addition to the wait result. That means consumers should not treat a wait response plus a notification as two distinct successful tasks. The V1 source accepts `send_input` with `interrupt=true` to redirect immediately or without it to queue input; `close_agent` closes the target and open descendants; `resume_agent` reopens a closed agent for further input/waits. Completed but unclosed agents count toward the V1 open-thread limit. These are distinct from merely ending a wait. [O4][O5]

V2 separates:

- `send_message`: queue-only, **does not start a new turn**;
- `followup_task`: triggers a turn if the target is idle, or delivers at message boundaries/after a pending tool call when already running;
- `interrupt_agent`: interrupts the current turn but leaves the agent addressable. [O4][O5]

The completion controller sends V2 terminal outcomes to the direct parent with `trigger_turn=false`. Its source explicitly calls delivery **best effort** and distinguishes queue acceptance from failure. Tests verify direct-parent queueing and that a dead direct parent is not silently replaced by the root as recipient. Therefore **do not assume an idle, finished Codex parent automatically starts another generation for every child completion**. Delivery into a running/waiting parent's mailbox and starting an idle parent are different operations. [O5][O6]

### Parent completion and restart

Child threads have independent execution/addressability, not a lexical stack frame inside `wait_agent`. The tests can shut down a live parent and still operate its child; a child completion to a dead parent is not rerouted automatically. This supports separating child lifetime from parent inference, but is **not** an unconditional guarantee that every Codex UI/process shutdown preserves live execution. [O6]

Source tests cover cold root reconstruction, lazy child loading, history restoration and later follow-up, as well as reopening open descendants after manager shutdown and respecting explicitly closed descendants. Those are useful persistence precedents. They do not establish exactly-once effects, a crash-proof acknowledged result inbox, or execution while the local application is closed. Status subscriptions explicitly describe live-runtime observation, not a durable continuation abstraction. Treat restart as history/tree reconstruction plus explicit subsequent execution, not resurrecting an old Rust future. [O6][O7]

## Claude Code and Claude Agent SDK

### Foreground, background, and changing the choice mid-flight

The current docs distinguish foreground subagents, which block the main conversation, from background subagents, which run concurrently while the parent/user continues. Current defaults differ from older accounts: main-conversation spawns default to background; `claude -p`/Agent SDK can choose to wait, while interactive surfaces need fork mode disabled to permit that choice. Setting `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` is different: it disables background execution and Ctrl+B conversion entirely. Do not turn a configurable/default behavior into a universal product rule. [C1]

Most directly relevant to the user's request:

- Terminal **Ctrl+B** or VS Code **Run in background** moves a task the parent is already waiting on to the background. [C1]
- SDK **`Query.backgroundTasks(toolUseId?)`**, documented since TS SDK 0.2.139, does the same. Select one tool call or omit the ID for all foreground tasks. Each affected call returns immediately with a running-in-background result; the **parent turn continues**; completion arrives later as `SDKTaskNotificationMessage`. This is explicitly not cancel-and-restart. [C2]
- The call resolves `false` when a specified tool-use ID matches no foreground task; disabled background tasks cause rejection. “Detach” needs a truthful outcome, not an assumed success. [C2]

Backgrounding can also affect capabilities: current docs describe a smaller built-in tool set for background subagents, with exceptions for forks/resumed foreground agents. Permission prompts from background children surface in the main session and identify the requesting child. These are Claude-specific permission/capability semantics, not inherent requirements of non-blocking execution. [C1]

### Several children and some/all joins

Docs explicitly support concurrent parallel research. Current documented default concurrency cap is 20 running subagents, configurable; it is not a cap on total invocations over a session, and the docs list exceptions. That cap is version-specific rather than a suggested Harness limit. [C1]

Foreground invocation returns `status: "completed"`; asynchronous invocation returns `status: "async_launched"` and its report in a later turn. A completion notification can be completed, failed, or stopped. SDK task events expose task IDs, tool-use IDs, and, in current versions, per-run IDs. Several queued completions can be handled together by one parent turn. [C2][C6]

**No general `wait_some(ids)` / `wait_all(ids)` host API was established in the inspected SDK reference.** Do not equate “several Agent calls” or a completion event with a documented all-child barrier. Foreground tool blocking and event-driven task tracking support the behaviors from which a host/coordinator can form a join. An explicit host-level some/all predicate would be a Harness design choice, not an existing Claude SDK signature. [C2][C6]

### A result frame need not mean all work is over

This is particularly clear in the open Python SDK source: [C4]

- Tracks `local_agent` and `local_workflow` lifecycle frames to avoid closing stdin while those tasks still need hook, permission, or SDK-MCP responses.
- Explicitly says a result frame ends one main-thread turn, **not necessarily the run**; a child can still run or a finished child's completion can still wake the parent for follow-up.
- Uses CLI-reported `session_state_changed` as well as task tracking because an empty live-task set can race with an undelivered/unfinished parent continuation.
- Deliberately does not use the `background_tasks_changed` snapshot as the sole ledger: a foreground child may be registered before it becomes backgrounded, without a second start event. It also excludes indefinite shells, monitors, teammates, and remote agents from the finite-task deferral set.
- Has bounded-between-turns handling for CLI state/compatibility, with explicit conditions under which the ceiling is not armed or does not cut off a tracked live agent.

These are source-backed control-channel lifetime decisions in that SDK. They are **not evidence of a durable product run database** or a reason to keep an actual model HTTP request open. They are strong evidence against collapsing “parent produced a final message/result” and “parent tree has no remaining work.” [C4]

For host integration, keep consuming the event stream rather than breaking at the first result. The agent-loop docs warn of trailing system events; the SDK source demonstrates the stronger child-work reason. Streaming-input sessions can accept more messages after ordinary results. Disposing the host/closing its control transport is a different boundary from one parent inference finishing. [C4][C5]

### Steering, stopping, and restart

Current docs expose direct child direction through `SendMessage`, with ID/name targeting and mid-task course corrections. Sending to a completed child can start a new background run under the **same agent/task identity**. Current task-event `run_id` distinguishes those runs, including late completion notifications from an earlier run. UI transcript access also permits direct user resumption where supported. Agent messages do **not** count as human permission approval. [C1][C6]

Cancellation has meaningful distinctions: model-issued `TaskStop` can be followed by a message resuming the stopped agent once it exits; a user stop in `/tasks` or SDK `stop_task` prevents model auto-resumption until the user resumes it. Therefore “stopped” alone is insufficient authority information for a durable scheduler. [C1][C2]

Child transcripts are stored independently under the parent session and can be resumed after restarting Claude Code and resuming that session, subject to retention and documented ID/name limitations. This guarantees resumable context, **not continued execution through process death**. Current SDK docs reset live background membership on CLI restart; restarted-process notifications and cloud-specific `reason: "worker_restart"` are documented, but the latter is expressly not a local `query()` field. Do not copy a cloud recovery claim into the local Harness MVP. [C1][C6]

## Pi: native primitives, shipped extension, and T3 are different layers

### Native Pi 1.5.0

The README explicitly says Pi skips built-in subagents. The SDK supplies: [P1][P2]

- `createAgentSession()`: one conversation/session, model/tool configuration, queues, extension runtime;
- persistent `SessionManager` history by default, with an in-memory option;
- `prompt()` resolving after the accepted run and automatic retries finish;
- `steer()` after the current assistant turn/tool calls, `followUp()` after pending run work, rejection of ambiguous concurrent `prompt()` calls;
- `waitForIdle()` versus `abort()` (stop active operation and wait for idle);
- `dispose()` aborting active work and tearing down the session integration.

`agent_end` is only the low-level agent-loop boundary: retries, recovery, compaction retry, or queued work can follow. `agent_settled` means Pi will not continue automatically. `turn_end`/`agent_before_settle` can request continuation; `agent_settled` is notification-only. One Pi turn sends one model request then executes tools. These exact distinctions are why a host should not mark a durable task complete from the first `agent_end`. [P2][P3]

An embedding host can own several sessions and not await a child session's prompt before doing other work. That is an architectural use of SDK primitives, **not a native Pi parent/child task service**, nor permission to call `prompt()` concurrently on the same busy session without choosing steering/follow-up. The host must own child IDs, scheduling, lifecycle, budgets, durable relationships, and result delivery. [P2]

### Shipped subagent example extension

The installed example registers one `subagent` tool. It starts separate `pi --mode json -p --no-session` processes with ignored stdin, streams progress, and returns final output when the subprocess exits. Its parallel mode accepts up to 8 tasks with 4 concurrent workers and awaits `Promise.all` over those workers. Chain mode awaits one child after another. [P4]

Consequences of this **specific example**, not limitations of all possible Pi extensions:

- Several child inferences execute concurrently, but the parent model waits on the enclosing tool until the whole parallel batch returns.
- Progress rendering is not completion delivery that lets the parent independently continue reasoning.
- There is no detach/background parameter, returned live-child handle, wait-some API, input channel for steering, resume operation, or durable scheduler in this example.
- Abort signal handling sends termination to subprocesses; the example does not establish guaranteed descendant/process-tree cleanup or transactional rollback.
- `--no-session` intentionally does not save standalone child sessions for later resume. Final results can be preserved in the parent tool details, but that is not recovery of a crashed in-flight child. [P4]

Thus the example alone does **not** satisfy the user's requested push-to-background interaction. Claiming that it demonstrates native Pi background delegation would conflate three layers.

### What extensions can add

Extensions can register tools/commands, send custom messages, request a turn when idle, and persist custom entries. `sendMessage` supports `steer`, `followUp`, and `nextTurn`; idle `triggerTurn` explicitly decides whether to start inference. Extension data can be reconstructed on session start. These are useful completion-notification and wakeup building blocks. [P3]

They are not a built-in durable job engine. Extension authors must define restart reconstruction, cancellation ownership, duplicate delivery, branch/history behavior, and cleanup. Pi warns that long-lived resources belong in session lifecycle hooks, not the extension factory; extensions run with full process permissions. A session replacement invalidates old session-bound contexts/subscriptions. Any scheduler built here must account for that lifecycle. [P2][P3]

### T3 app-owned delegation: observed contract

The following is a dated snapshot of tool descriptions/schemas exposed to this research session, **not an implementation trace**. No listed execution tool was called. [T1]

| Tool/field | Exposed semantics |
| --- | --- |
| `delegate_task({task, mode?: "async" | "wait", timeoutMs?})` | Starts one child; async is default. Wait mode waits in this call; question or approval ends the wait; wait timeout does **not** cancel the child. Separate `taskId`, `childThreadId`, `childRunId`, terminal-run fields, `workState`, `hasPendingChildRuns`, and optional `next` are returned. |
| `task_status({taskId})` | Reads a task started by this caller. `workState` is `working`, `waiting_for_children`, or `result_available`; a finished turn with live child work is not done. Reading marks its result delivered. Instructions explicitly disallow polling a running child before its completion wakeup. |
| `t3_thread_wait({threadId, runId?, untilDelegatedWorkSettled?})` | Wait for one thread run, latest by default; idle returns immediately. Optional settled mode includes delegates/background tasks and completion settlement, with `resultRun` identifying the final-answer run. A timeout with `next` directs ending the current turn; without `next`, another wait is permitted. |
| `t3_background_wait({taskIds, waiting?})` | Claude-only activity integration: keeps thread Working while background work runs. `waiting=false` clears that indication and does not stop tasks. Accepts multiple task IDs. It is not itself proof of a durable some/all join. |
| `t3_thread_send({threadId, message, mode?})` | `auto` starts/steers/queues, `queue` starts a later turn, `steer` updates mid-turn, `restart` interrupts/restarts. Sending to a finished child thread creates a fresh task and returns its ID. Calling this is prohibited for fresh review/re-checks by this session's policy; the protocol capability and policy are different. |
| `task_cancel({taskId})` | Stops running/queued turns, delegates, PR watches and later runs in the child thread. A finished task keeps its result. Broader scope than interrupting only one provider request. |

T3's prompt policy says follow a running delegate's `next`, do not poll, and do not mistake a terminal inference turn with pending children for completed work. This permits asynchronous concurrent children and parent work between dispatch and wakeup at the exposed-contract level. It does **not** prove that every wait terminates a durable run. [T1]

No general **mutate-existing-task-to-background** operation was established in the inspected delegation schemas. Starting with `mode:"async"` is supported; a wait timeout is non-cancelling. Whether an in-progress `mode:"wait"` call can be released interactively, how wakeups survive a server crash, which inbox/cursor is persistent, and exactly how some/all waits settle require T3 source or separately authorized runtime verification. T3's tools are app-owned orchestration accessible through Pi, not native Pi APIs. [T1][P1]

## Comparison of the requested behaviors

| Behavior | Codex pinned source/docs | Claude Code / SDK | Native Pi / example | T3 exposed contract |
| --- | --- | --- | --- | --- |
| Foreground wait | Separate async wait tool; V1 final-status / V2 activity wait | Foreground blocks parent; SDK can explicitly wait | Await `prompt`; example awaits subprocess/batch | `mode:"wait"` and thread wait |
| Several children; parent other work | Spawn handles first, join later | Background children run concurrently | Host can build multi-session scheduling; example only concurrent batch behind one tool | Async tasks; parent may work until it chooses to yield |
| Initially foreground, then background | Release/finish a separate wait; no verified dedicated conversion UI | Explicit Ctrl+B/UI/`backgroundTasks` without restarting child | Not in shipped example | No verified live wait-conversion operation |
| Wait for some / all | V1 some-final; V2 any activity; all requires coordinator predicate | Task events + foreground waits; no established general host some/all API | Example all-batch only; host-defined otherwise | One-task/thread waits; settled-tree option; no established general some/all primitive |
| Completion after parent inference ends | V2 queue-only to direct parent, not automatic idle-parent inference | Later notifications/follow-up; SDK keeps control transport alive for owed work | Extension/host must deliver/trigger; `agent_settled` covers only that Pi session | Terminal run plus pending-child/work-state fields; wakeup/`next` contract |
| Steering/cancel | V1 send/interrupt/close; V2 message/follow-up/interrupt | Message child; user stop distinct from model stop | Native session steer/abort; example child stdin ignored | Send modes versus task-scoped cancel |
| Restart durability established here | Persisted history/tree and explicit follow-up, not exact-once delivery/effects | Resumable transcripts; process restart not live execution | Persistent sessions; example disables child session files | Unknown from schemas alone |

Cells summarize the individually cited sections above. “Not established” is an evidence limit, not a claim that the feature cannot exist.

## Semantic alternatives for Harness — decision still belongs to the user

### Common interface direction supported by the evidence

These are **recommendations to evaluate**, not accepted requirements beyond the user's explicit foreground/background/concurrency request:

1. **Spawn returns stable child/task identity.** A foreground convenience operation can combine spawn + wait, but must not hide the handle needed for detach, steer, and cancel. Backgrounding changes the parent's dependency/attention state, not the child's objective, workspace, or identity. Claude supplies a direct precedent; Codex supplies the decomposed alternative. [C2][O2][O3]
2. **Specify joins explicitly:** child run/attempt identities, `some`/`all`/possibly quorum, whether terminal failures satisfy the predicate, a timeout, and whether cancellation propagates. Timeout or stopping the wait must not silently cancel children. “All” should normally mean a captured set, not a moving target that grows with future spawns. These are proposed details, not facts about all surveyed harnesses.
3. **Persist completion before scheduling parent consumption.** Distinguish `child terminal`, `result available`, `notification queued`, `parent consumed`, and `parent follow-up settled`. Claude's SDK race handling and T3's separate settlement/read-delivery concepts demonstrate why one boolean is insufficient. Codex's explicitly best-effort delivery shows why stronger durability must be built rather than assumed. [C4][O5][T1]
4. **Address direct child steering independently of the parent's current wait.** Serialize accepted instructions into the child's durable history with provenance and target-run identity; do not confuse an agent's message with human authorization. If a child finishes while a steering message is in flight, explicitly choose reject, queue-for-next-attempt, or new run. [C1][C6][H1]
5. **Keep cancellation scopes separate:** release a wait, cancel current child attempt, cancel the delegated objective/tree, or terminate the host. Preserve uncertain external effects and approvals. Harness already records interrupted/unknown outcomes and avoids blindly replaying actions. [H1][M]

### A. Suspend the same logical durable run

A parent that has no independent work records its wait predicate and continuation, becomes `waiting_for_children`, and releases its inference/execution resources. Child completion makes that **same nonterminal durable run** runnable again; the next model step reads committed results. A foreground wait can present as one operation while the scheduler is fully yielding.

- **Benefits:** cohesive run-level objective/accounting; no artificial new run for every child wait; close fit to foreground dependency reasoning and the user's withheld decision.
- **Costs:** durable run state must distinguish runnable, suspended, approval-blocked, terminal, and recovery-interrupted. “Run duration” cannot mean only active inference time. Recovery reconstructs a continuation from records, not a suspended Rust stack.
- **Background action:** remove/release the foreground join and schedule independent parent work; keep child execution and task identity unchanged.
- **Restart:** restore wait-set and consumed-result cursor; reevaluate predicates; do not replay uncertain tools. Human-input waits still follow the map's separate approved later-run rule. [M][H1]

### B. End an execution attempt; wake a later run

A wait ends the current **execution attempt/run** with a non-final delegated-task disposition. Persist a subscription and parent task; on the selected completion predicate, enqueue a new parent run with durable context.

- **Benefits:** short, terminal execution records; no suspended run to lease; natural explicit recovery and budget accounting per attempt.
- **Costs:** requires a durable parent task/objective above runs, otherwise “finished run” falsely means “finished task.” More run links, notification deduplication and continuation context; every wait becomes a run boundary even when the foreground UX looks continuous.
- **Background action:** continue independent work in another attempt or keep the current attempt running before it commits a wait. The UI must not report task completion merely because one run ended.
- **Restart:** reconstitute subscriptions and enqueue at most one continuation for the relevant child attempts. Use an idempotent scheduling key; never equate exactly-once scheduling with exactly-once external effects.

This is a valid alternative to present, **not something established-harness behavior forces or the user has approved**. The map applies this policy to human input only. [Q][M]

### C. Bounded in-process wait, durable fallback

Use an ordinary yielding foreground wait for a bounded interval; after that, persist a suspended wait (A) **or** end the execution attempt and persist a later-run continuation (B). Allow user-triggered backgrounding at any time without waiting for the bound.

- **Benefits:** simple/low-latency short waits; familiar UX; bounds held resources without burning parent inference; compatible with T3's non-cancelling wait timeout precedent. [T1]
- **Costs:** two paths and a transition race. Child completion concurrent with timeout/detach must resolve into one durable continuation, not a lost result or duplicate parent run. The policy after the threshold must be explicit, not an accidental SDK timeout outcome.
- **Restart:** treat an interrupted in-memory wait as a recovery case for the saved predicate, not evidence that the child failed.

**Recommendation among alternatives:** retain A and C as serious candidates alongside B, rather than treating “no inference is running” as an argument for B. All three can deliver foreground waiting, push-to-background, several concurrent children, and parent independent work if task identity and result delivery are separate. The decision should turn on the intended meaning of *durable run*, accounting/visibility, and recovery complexity—not on a false claim that established harnesses require a particular run boundary. [C2][C4][O3][T1]

## Durability questions and verification work still needed

Before approving a policy, answer:

- Is a durable run an objective-wide record, an execution attempt, or a scheduler activation? Which identity owns approval, routing, budget, and final answer?
- When parent inference stops with live children, does it remain nonterminal, become a terminal attempt with a live task, or wait only in memory? Which TUI state tells the truth?
- Does finishing a parent response leave children running? Does explicit task cancellation cascade? What about human stop versus model-directed interruption?
- Does a late completion wake an idle parent automatically, enter a visible inbox awaiting action, or wait for a join? What changes for cancelled parents?
- Are some/all waits keyed by child task or exact attempt? How are failure, user questions, approval pauses, nested children, and replacement attempts handled?
- How does direct human steering invalidate prior approval or conflict with parent-issued instructions? No surveyed product's convenience messaging should be imported as automatic authority.
- T3-specific unknowns: executing server version/commit, wait-to-background UI/API, persisted wakeup/acknowledgement contract, crash reconciliation, and exact settlement predicates. Schemas do not settle these.
- Claude-specific limit: documented defaults/SDK APIs are versioned; closed CLI scheduling/crash internals were not inspected. Codex-specific limit: V1/V2 feature/model selection and client-specific process shutdown semantics were not runtime-verified.

Suggested **future temporary external probes**, not performed here: start two finite children; wait for one; detach the other; do parent work; deliver out of order; finish parent inference before child completion; steer the child; distinguish timeout from cancellation; cancel a parent/tree; crash before/after child terminal commit and before/after parent receipt acknowledgement; restart and verify one continuation with no blind tool replay. Include a child that requests permission and an indefinite watcher to ensure “all finite children settled” does not mean “every process in the session has exited.” This follows Harness's external-probe/no-permanent-test standard. [H1][C4]

## Validation performed

- Read the ticket and map in full through Plane; made no state or map changes.
- Verified exact release/tag commits and compared the required installed Pi documents/example against their pinned upstream copies.
- Read the cited wait handlers, completion routing, SDK lifecycle logic, and relevant restart tests; did not execute those upstream tests or exercise delegation.
- Checked report citation identifiers and staged whitespace with `git diff --cached --check`. Product build/test/check commands were not run because only this research document changed.

## Sources

[Q]: https://app.plane.so/reilley/browse/HARNESS-12/
[M]: https://app.plane.so/reilley/browse/HARNESS-1/
[H1]: https://github.com/Reilley64/harness/tree/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4
[O0]: https://github.com/openai/codex/releases/tag/rust-v0.167.0
[O1]: https://developers.openai.com/codex/subagents
[O2]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers/multi_agents/spawn.rs
[O3]: https://github.com/openai/codex/tree/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers
[O4]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers/multi_agents_spec.rs
[O5]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/agent/control/completion.rs
[O6]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/agent/control_tests.rs
[O7]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/agent/control/watch.rs
[C0]: https://github.com/anthropics/claude-code/blob/0ed94698aa0aeee471465fc9c867c2e8434c4d68/CHANGELOG.md
[C1]: https://code.claude.com/docs/en/sub-agents
[C2]: https://code.claude.com/docs/en/agent-sdk/typescript#backgroundtasks
[C3]: https://github.com/anthropics/claude-agent-sdk-typescript/blob/49dd8ef10dd5cb012dc9fe10703376489850c36c/CHANGELOG.md
[C4]: https://github.com/anthropics/claude-agent-sdk-python/blob/2209f2c134fee914c0f75733b631a5fc238ca846/src/claude_agent_sdk/_internal/query.py
[C5]: https://code.claude.com/docs/en/agent-sdk/agent-loop
[C6]: https://code.claude.com/docs/en/agent-sdk/typescript#sdktaskstartedmessage
[P1]: https://github.com/earendil-works/pi/blob/f9a26365568127461c47a07a37dcb42c140a3ac5/packages/coding-agent/README.md
[P2]: https://github.com/earendil-works/pi/blob/f9a26365568127461c47a07a37dcb42c140a3ac5/packages/coding-agent/docs/sdk.md
[P3]: https://github.com/earendil-works/pi/blob/f9a26365568127461c47a07a37dcb42c140a3ac5/packages/coding-agent/docs/extensions.md
[P4]: https://github.com/earendil-works/pi/tree/f9a26365568127461c47a07a37dcb42c140a3ac5/packages/coding-agent/examples/extensions/subagent
[T1]: #t3-app-owned-delegation-observed-contract

### Source navigation notes

- **[H1]**: `GLOSSARY.md`, `docs/conversations.md` (Scope, Workflow, Persistence and crash recovery, Steering), `CODING_STANDARDS.md` (checkpoint-before-effect, uncertainty, verification). No existing durable multi-agent run contract is claimed from this baseline.
- **[O1]**: Orchestration and thread controls; configuration concurrency limits. Markdown was fetched via `/codex/subagents.md`; documentation may redirect to the current OpenAI learning site.
- **[O3]**: exact V1 [wait handler](https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers/multi_agents/wait.rs), `handle_call`/`wait_for_final_status`; exact V2 [wait handler](https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers/multi_agents_v2/wait.rs), `handle_call`. These two protocols are intentionally not collapsed.
- **[O4]**: `create_send_input_tool_v1`, `create_send_message_tool`, `create_followup_task_tool`, both `create_wait_agent_tool_*`, `create_close_agent_tool_v1`, `create_interrupt_agent_tool_v2`. V1 [send-input implementation](https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers/multi_agents/send_input.rs) was also read; V2 [follow-up implementation](https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/src/tools/handlers/multi_agents_v2/followup_task.rs) supplies `MessageDeliveryMode::TriggerTurn`.
- **[O5]**: `notify_parent_of_terminal_turn`, including best-effort comment and `trigger_turn=false`.
- **[O6]**: `multi_agent_v2_completion_ignores_dead_direct_parent`, `multi_agent_v2_completion_queues_message_for_direct_parent`, `resume_agent_from_rollout_reopens_open_descendants_after_manager_shutdown`, `resume_agent_from_rollout_does_not_reopen_closed_descendants`. Also read [cold-restore suite](https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/core/tests/suite/multi_agent_restore_tests.rs), `cold_root_resume_overlaps_child_reads_and_applies_identities_in_graph_order`. Tests were inspected, not executed.
- **[C1]**: Run subagents in foreground or background; Concurrent subagent limit; Resume subagents; What subagents inherit. [SDK subagent guide](https://code.claude.com/docs/en/agent-sdk/subagents) corroborates `async_launched`/`completed` and `AgentDefinition.background`.
- **[C2]/[C6]**: same TypeScript reference, sections `backgroundTasks()`, Query object, `SDKTaskNotificationMessage`, `SDKTaskStartedMessage`, `SDKTaskUpdatedMessage`, `SDKBackgroundTasksChangedMessage`, and result origin. Per-task `run_id` requires SDK 0.3.292+; background membership docs require reset after CLI restart. The general API reference also documents `stopTask`.
- **[C4]**: Python package [version manifest](https://github.com/anthropics/claude-agent-sdk-python/blob/2209f2c134fee914c0f75733b631a5fc238ca846/pyproject.toml); query source `DEFERRING_TASK_TYPES`, `_track_task_lifecycle`, `_on_session_state`, `_maybe_end_run`, `_reopen_run`, `wait_for_result_and_end_input`. This is public SDK source, not a claim to have seen the CLI implementation it communicates with.
- **[P4]**: `index.ts`, `mapWithConcurrencyLimit`, `runSingleAgent`, `SubagentParams`, tool `execute`; accompanying README. Installed SDK file SHA-256: `d9034ccbf2219a58712d8dd4fcdf47e687d77131e71dd5d119d8a71e9bd2a2e8`; example `index.ts`: `1d11858d163e2a3a258af51c1923fc55ad825813acd6224462fd25ad61f5401b`.
- **[T1]**: source is the exposed `mcp__t3-code__delegate_task`, `task_status`, `t3_thread_wait`, `t3_background_wait`, `t3_thread_send`, `task_cancel` descriptions and schemas plus session orchestration instructions on the verification date. The table preserves the relevant observed contract because no public, version-pinned T3 source was established. It is not independent proof of server enforcement or durability.
