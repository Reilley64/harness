# Using conversations and navigation

## Conversations

`useConversation` owns optimistic streaming display, explicit cancellation,
error reconciliation, and the last authoritative snapshot. Rust owns the turn/tool
loop and steering; the hook sends instructions rather than cancelling/resending.
An instruction-adoption acknowledgement updates only its same conversation target
at a non-older revision; a delayed reply cannot erase a newer completed turn or
regress the map's run state. A stale initial Griller-creation acknowledgement is
rejected before adopting instructions, without replacing newer history; inspect
again to adopt against the current conversation. Durable commits are not rolled
back by these presentation guards. `useConversations` keeps the selected controller and retained background
controllers separate. It opens existing agents through read-only conversation
views, restores their model snapshots on explicit selection, and invalidates
superseded navigation. While a conversation open is pending, the previous view
remains visible but its editor and commands are inactive. Following editing input
is ignored, not submitted to the previous agent or queued for replay. Escape
cancels the pending open and returns to the map. Only the current successful open
changes the selected conversation and model; failure restores editing with its
error visible. Selection does not cancel or replace another controller. Interruption acknowledgements require the same
selected actor, current navigation ownership and a non-older revision; a durable
acknowledgement cannot overwrite newer history or another actor's context/model.
The first send creates a session; opening a picker or choosing a model does not.
After failure the hook reloads saved history so it does not mistake optimistic
text for persisted messages or automatically resend a failed turn.

The transcript renders message text and partial statuses in a bounded native Ink
viewport. Each assistant text section has top/bottom padding, including prose
between tool calls and the final summary. Message sections extend to both terminal
edges;
there are no user/assistant headings, conversation ID banner, paging hints, or
model-selection notices. Per-reply provenance stays in durable metadata. Its Pi-inspired styling gives user
messages blue-tinted sections, keeps assistant replies on the terminal background,
and uses amber/red bands for interrupted/cancelled/incomplete/refused and failed
replies. Empty pending assistant messages do not add a duplicate thinking panel;
the prompt accent is violet.
Native Ink background/padding props paint each visible row, including clipped
sections during paging. `services/transcript.ts` formats stable, sanitised rows;
`MessageLine` renders them, and `theme.ts` centralizes the palette. Panel foreground
and background colours are paired; assistant body text uses the terminal default.
Labels/status text remain meaningful with colours disabled. `wrap-ansi` measures wrapped lines for
PgUp/PgDn paging; display primitives still sanitize control characters. The
wordmark depends on actual visible messages, not commands or picker activity.
During a reply the prompt stays focused and editable. Enter submits an acknowledged
instruction to Rust's control channel. It continues the same logical turn, without
duplicating the user message or replaying unfinished prose. Running tools finish
before remaining batch calls are skipped. Escape closes/back-navigates only. Rejected submissions
retain their exact submitted whitespace. The latest rejection restores directly to
an empty editor when no rejected drafts are already queued. Otherwise it enters a
session-local rejected-draft queue without replacing newer text, even if newer
submissions have already been sent. A visible notice counts queued drafts; clear
the editor, then press Ctrl+R to recover the oldest known rejection in submission
order. Recovery only fills an empty editor and never sends automatically; Enter
sends the recovered text normally. Out-of-order failures are sorted in the queue,
not concatenated, and accepted sends do not clear it. This also covers transport
errors, cancellation before acceptance, missing-model and oversized rejections.
Queued drafts are held in memory until recovered or the screen exits, not saved to
conversation history. Late send results do not update draft state after unmount.
Completion does not erase text being edited. Slash commands other than quit
require explicit Stop first, except navigation and Stop itself.

Tool notifications update stable call IDs in place. Padded grey panels become
muted green on success or muted red on failure/Unknown; cancelled/skipped calls
stay grey. Parent-turn cancellation does not recolour successful tools. Tool
headings are bold and show paths, requested line ranges, timeouts, and other
relevant parameters instead of success labels or elapsed durations. Non-success
states remain explicit. Read/write code uses extension-based syntax colors; edits
use unified-diff coloring. Successful writes include a preview of the written
content. Highlighting is cached per immutable call, bounded, and falls back to
plain text for unknown languages or unsupported output. It never embeds terminal
escape sequences from tool output or changes panel backgrounds.
Ctrl+O expands/collapses output; resume reconstructs the same panels from snapshots.
Mouse wheel, PgUp/PgDn, and Shift+Up/Down scroll during both generation and idle.
The viewport fills its measured available height, bottom-aligns short transcripts,
and preserves its anchor while output streams; reaching the bottom resumes
following new output. A screen-owned viewport hook retains independent anchors
and tool-expansion state by durable conversation identity for the current UI
session. Switching Griller/CoS, opening pickers or visiting an empty conversation
cannot reuse another actor's numeric anchor or erase the previous reading position.
This is in-session state, not durable scroll restoration after process restart. Escape preserves the anchor while navigating. Native terminal selection uses
Shift+drag while mouse reporting is enabled. Mouse modes are restored on unmount.
Unchanged immutable message objects retain cached row layouts; each message keeps
only its current width/expansion layout in a weak cache. Visible rows are memoized,
wheel events move one row, and Ink renders at up to 60 FPS. See
[tool calls](../tool-calls.md) for permissions, continuation, limits, and verification.
The prompt reserves its content row even when empty/unfocused and cannot shrink
under layout pressure. Multiline drafts expand within a terminal-sized viewport;
only rendering is clipped, and submission sends the entire draft unchanged.
Oversized submissions fail visibly and preserve the draft. A starburst
indicator occupies a separate full-width row outside the scrolling viewport, with
the same horizontal and vertical section padding as chat. It remains immediately
above the prompt, followed by unbracketed elapsed time. The prompt border and
active thinking text share a smooth three-minute rainbow cycle through a subdued
Pi-dark-inspired palette; one 250 ms clock updates only its subscribers, not the
screen or transcript. Picker borders remain static. Theme foregrounds, user/tool
backgrounds and syntax colors follow Pi's built-in dark palette rather than ANSI
magenta; this is not Pi's adaptive system-theme implementation. It switches to grey
`Worked for …` when the send/steering chain finishes, including
cancellation or failure. Its monotonic elapsed time covers session preparation,
generation, and durable finalization, not provider-reported reasoning time. The
spinner timer stops when idle or unmounted; the shared color timer runs while an
active prompt or thinking indicator is visible and stops after its last subscriber
unmounts. Startup takes raw input ownership before rendering and enables Kitty
keyboard reporting without a capability query, avoiding echoed probe replies;
unsupported terminals retain Alt+Enter/Ctrl+J, and Ink restores reporting on exit. Tab or right arrow accepts the
prompt's ghost command completion; normal input navigation remains when there is
no completion. Terminal selection remains terminal-controlled: Ink cannot
selectively make only the background ASCII wordmark unselectable.

The footer has two rows: the bound workspace and its current Git branch above the
controls, and cumulative provider-reported input/output tokens above the model
name. The branch appears as `(main)` beside the path, or `(detached <short-id>)`
for detached HEAD. Read-only, bounded Git queries refresh about every two seconds
against the bound workspace (including resumed workspaces and linked worktrees),
not the Harness checkout. Missing Git and non-repositories show no branch label;
queries are cancelled on workspace changes/unmount. Long labels truncate within
the existing footer row. `ctx` is the latest
reported request's input plus output tokens divided by the selected model's
optional catalog `context_window`; it is not cumulative token billing or account
quota. It excludes unsent drafts and changes since that reported request. Unknown
usage/capacity displays `—`; missing generation reports and historical gaps mark
known totals with `≥`. Reported superseded or cancelled generations still count;
steering does not itself make complete reports partial.
Per-step reports update live and are restored from durable history. The same
footer also shows [provider waits](../provider-waits.md): timed retry/quota,
unknown-reset `/resume`, or exhausted `/retry extend <n>`. `/usage`,
`/usage run <id>` and `/usage spec <work-id>` read reported/estimated/unknown
accounting without inference, including while waiting. Live quota Resume and
trusted explicit extension update the suspended controller, not a second prompt.
After restart `/resume` still uses ordinary recovery; idle `/retry abandon`
retains the episode history and clears no independent gate. No local
character-to-token guesses or fabricated model limits are used.

The entry point accepts `--resume <conversation-id>` and loads it before rendering.
Invalid/missing/nonexistent identifiers fail visibly. The saved model/effort seeds
the picker without requiring the catalog. No arguments detect missing applicable Setup outputs and open the saved Setup
actor when incomplete, otherwise the saved repository Griller; an absent actor
renders an empty view without creating history.
When that repository actor does not exist, startup restores the last model/effort through
`_harness/model-selection/read`, without credentials, catalog access, or history.
Explicit resume takes precedence and does not overwrite this default just by
reading a conversation. A missing preference is normal; an invalid one produces
a nonfatal warning and can be replaced by choosing a model. Picker commits are
serialized through `_harness/model-selection/update`; exit waits for these saves.
`/open <conversation-id>` opens an existing saved conversation;
`F3` or `/griller` selects the repository Griller. Existing-agent navigation is
available while another agent works; it does not resume, restart or stop that
agent. The creation map is the durable-agent navigator, not a general picker for
unassociated conversation history.

The ACP service translates standard sessions/prompts/updates/cancellation, parses
safe conversation metadata, and never receives credentials. On exit it cancels
active replies and waits for their durable finalization before closing the
subprocess. See [conversations](../conversations.md).

## Managed Research inspection

F6 or `/children` opens the durable inspector without submitting or clearing the
parent editor. Select with arrows; Enter opens that child's actual saved/live
history read-only, refreshed without recovery or another execution controller.
PgUp/PgDn/Ctrl+O retain native history behavior. Escape returns to the inspector
then the parent; it never stops a child. Background updates do not steal selection,
title, root draft or per-conversation viewport. Reports page independently.

Space selects fixed members; A/L capture named any/all joins; B backgrounds the
selected captured join without restarting workers. R refreshes, O reads the
bounded untrusted report and X requires confirmation before scoped child Stop.
N/Escape cancels that confirmation. Missing/oversized authority remains visibly
blocked, not an empty or settled child view. Reports/receipt/follow-up facts do
not approve work.

`/children guidance <intent>` inspects the pending Research snapshot. V explicitly
adopts the exact available hash against the inspected parent revision; accepted
snapshots cannot be replaced. `/children dispatch <JSON>` submits the bounded
brief described by [ACP](../acp/children.md). These controls remain available
during parent work without cancelling/resending it. Exit waits for pending child
operations and owned backend draining before closing stdio.

`/children steer {"executionId":"<owned execution>","text":"<instruction>"}`
submits a distinct targeted instruction, not another parent message. The inspector
shows its persisted order and effective-boundary acknowledgement, plus the latest
accepted/applied instructions for the selected child. Rejected commands retain
their editor text. No cancellation/resend, assignment restart or later-run enqueue
implements this control.

`/children answer {"executionId":"<owned execution>","requestRun":"<input run>","text":"<answer>"}`
answers that exact retained child request through the typed backend interface.
`/answer {"requestRun":"<input run>","text":"<answer>"}` answers the selected
durable agent. Answers start later runs, not replacement agents or assignments.
The retained-result inspector opens after the answer; Esc closes it before chat
editing or F2 navigation. Ordinary chat/update text cannot answer the question,
approve scope or settle work.
`/children scope {"executionId":"<owned execution>","id":"<request UUID>","text":"<criteria>","replaces":null,"supersedesRun":null}`
persists a targeted amendment. Replace null with the exact predecessor and obsolete
input-run identities when applicable; prose cannot silently supersede them.
The acknowledgement shows its assigned exact version.
`/children approve {"id":"<request UUID>","version":"<exact version>","approvalId":"<approval UUID>"}`
is the separate trusted approval control. Inspect first; stale versions fail.
V in the child inspector pages the full criteria and approval identities;
Left/Right there selects retained scope versions. Escape navigates only.
Commands use the existing rejected-draft queue, so delayed acknowledgements cannot
erase a newer draft or another agent's editor.

`/children resume <execution UUID>` explicitly resumes an eligible recovered
child after its owner has been explicitly resumed. Stop and Unknown tool outcomes
cannot be cleared by this command. Recovered children show a Resume-required flag.
Rejected answers retain editor text. Inspector Stop remains available while an
answer is executing; Escape still navigates without cancelling it.

Child Stop traverses [retained execution ownership](../managed-dispatch.md#ownership-scoped-stop).
The inspector remains a read-only history/report surface. The creation map can
open the same existing child as an editable agent context with independent
in-session drafts and scroll state. Neither surface provisions managed Implementer
workspaces or expands runtime recovery authority. [Managed dispatch](../managed-dispatch.md) owns runtime authority,
recovery, limits and diagnostic retention.

## Agent contexts and creation-map navigation

The native map reads the [durable-agent inventory](../acp/creation-map.md), not a
static role roster or an execution tree. One node identifies an actual recorded
agent, independently of its runs. Human answers, model changes, stopped/completed
runs and reloads do not create replacement nodes. AgentId and ConversationId are
separate identities; map selection uses the former and opening/Stop addresses the
latter. Unassociated pilot conversations are not invented into agents.

Recorded creator arrows can cross execution-root boundaries: a Griller-created
CoS can own Research work without being in the Griller's Stop subtree. Missing
creator metadata remains unknown. A terminal owned-assignment outcome is shown as
completed/failed/stopped even when its agent's generic next-turn gate is
needs-human. Restart/unknown authority still takes precedence. Without live
coordination or an owning execution, run status is explicitly labelled saved;
absence of a recorded run says `no saved run`, never fabricated readiness.
Pending-result and required-follow-up counters come from retained owned-child
receipts. Completed child work does not settle the parent or approved spec.

Arrows pan the clipped canvas; Tab/Shift+Tab selects visible agents and centres a
selection when necessary. C collapses/expands the selected creator branch without
changing agents, executions or captured membership. Home centres the selected
node. Growth preserves a visible card's cell anchor when follow is off, rather
than moving the viewport to every new agent. New-agent counts accumulate since
explicit acknowledgement; N reveals the latest observed addition and acknowledges
the batch. F explicitly toggles follow of new additions. Follow pans only: it does
not select an agent, change the chat/title or erase a draft. Manual pan does not
silently toggle the user's follow preference.

The selected conversation retains its own draft, rejected-draft queue, line-ID
scroll anchor and tool expansion for the current frontend session. Reading another
agent leaves background controllers mounted. Returning to CoS while A works can
accept another instruction and dispatch B without restarting A. This is not
persistent draft/viewport restoration across frontend restarts.

Plain Left from the empty focused editor invalidates any pending conversation
open before showing the map; whitespace, modifiers,
paste and picker input retain their editing meaning. Function-key-shaped bytes
inside bracketed paste never execute navigation, including fragmented markers
and large paste chunks. Paste tracking survives picker activation changes.
The [shared input owner](implementation.md#terminal-input) applies the approved
150 ms legacy Escape idle window before forwarding complete key/paste frames to
Ink. Complete unambiguous frames do not wait for that timer. Synchronous editing
state and committed-context acknowledgement keep coalesced space-plus-Left and
navigation batches out of hidden editors. F2 toggles chat/map; F3 selects Griller
without stopping the current agent. Enter opens the selected existing node.

A single mouse click selects a real card. Opening requires two matching
press/release pairs on the same card within 400 ms. Empty space, interrupted
gestures and reports begun before a geometry change cannot open another agent.
Public Ink measurements accumulate ancestor offsets in the full alternate-screen
live region; parent-relative offsets are not terminal coordinates. Wheel reports
move the canvas three rows each and remain cumulative when coalesced; they do not
select a card. Resizing, panning and growth invalidate identity-targeted gestures,
not safe relative wheel movement.

Titles are exactly `harness - Griller`, `harness - Setup`, `harness - Research`,
`harness - Chief of Staff`, `harness - Children` and `harness - Agent map`; an explicitly
loaded unassociated pilot uses `harness - Conversation`. Only static context labels
enter title escapes. Background updates cannot change selection/title. Title,
mouse and raw/alternate-screen modes pair with cleanup. Conversation controllers and per-conversation viewport state survive map visits;
the transcript need not remain mounted to retain its line anchor. Unsaved drafts
are untouched. Anchors identify the viewport's last visible logical line, so
adding/removing a chrome row may expose an extra older line without moving that
anchor.

Escape closes a picker, Stop confirmation or authentication flow, or navigates
between conversation and map; it never stops a run. Use `/stop` or Ctrl+X on the
map for selected-scope Stop confirmation. Y/Enter confirms; Escape/N closes without
cancelling. For managed agents, the confirmation identifies the owned execution
subtree including background descendants, excluding siblings; the hook requests
[durable ownership-scoped Stop](../managed-dispatch.md#ownership-scoped-stop)
before acknowledgement. It also works while the selected parent is idle.
Setup and unassociated pilot conversations retain selected-run cancellation.
A failed durable fence is visible even if local cancellation succeeds; no empty
subtree acknowledgement is invented. Confirmation explains that completed effects
remain and uncertainty is not rollback. Ctrl+X inside the editor cuts its current logical line, not the
run. ACP cancellation remains available without adding a human approval gate
to backend/tool authority.

Interrupted work displays Resume-required state. This includes an idle parent
whose background ownership was lost: a typed answer alone cannot bypass restart.
At an ended input boundary, `/resume` restores the same question without inference;
use its matching answer afterwards. Live owners in another backend are not treated
as dead just because the inspector reads them.

Interrupted active work displays Resume-required state. `/resume` explicitly continues
the same run. For uncertain effects, `/resume inspect` acknowledges the interruption
without generation, retaining Unknown results and permitting a new inspection
instruction. Reopening, map navigation or ordinary prompts cannot start recovered
work. See [conversations](../conversations.md).

External stock-launcher PTY probes cover 80×24/50×18, exact title bytes, actual
backend kill/restart/Resume, empty versus whitespace/modified Left, Unicode
multiline paste/draft retention, F2/F3, split SGR clicks, Escape during inference,
confirmed Stop and normal terminal-mode restoration. These byte-report
observations are not a human terminal-emulator visual signoff.
