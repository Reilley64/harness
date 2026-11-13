# Versioned role guidance

## Implemented boundary

Griller's real `create_message`/Resume coordinator captures instructions before
inference and passes the captured text through the stock provider prompt path.
`ReadInstructions` is a closure-backed application port; its matching adapter
resolves the embedded bundle, explicit configuration and repository documents.
`read_guidance` inspects without inference or mutation; `update_guidance` explicitly
adopts an inspected candidate under the conversation lock and checked revision.
Domain values remain Serde-free. Each conversation adapter owns its own records.

Griller, [Setup](repository-setup.md) and the approved-work Chief of Staff execute
through the shared loop. [Managed Research](managed-dispatch.md) resolves its
Research closure through that same machinery. [Managed Implementers](managed-workspaces.md)
execute their implementation closure. Reviewer, review-axis and Retro assets remain
bundled for later assignment owners;
inspection is not an agent roster, dispatch API or creation relationship. Native
Jev Decider has a separate typed-decision contract, not a conversational/tool role.
All conversational roles use shared configured tools. Guidance cannot grant scope,
settle work, reset allowances, authorize writes/merge or resolve Unknown effects.
Bash/MCP are OS-user trusted; source worktrees do not isolate services or secrets.

Child execution/joins/steering and completion/continuation belong to
[managed dispatch](managed-dispatch.md), not this resolver. Compaction and provider
fallback are not implemented by this module. Missing glossary/ADR files are not unconditional Setup triggers.

## Bundle and provenance

`infrastructure/src/read_instructions/bundle/` contains Harness-owned adaptations,
versioned `harness-main-flow-1`. `bundle.rs` is the embedded catalog and dependency
index. Each asset exposes its exact source, content version, required asset IDs,
full body and pinned upstream source links. Local/repository override content
never inherits an upstream identity simply because it replaces a bundled ID.

The audited upstream is `mattpocock/skills` 1.3.2 at
`4ae2317ddb22b046e26455e4f9c9b3aaaf62212d`. Its MIT notice is retained in the bundle.
The product neither reads nor changes personal/global installed skills.

Adaptations retain Main Flow's frontier interview, domain modeling, public-seam
specification, dependency-linked ticketing, primary-source research, independent
review axes and environment Retro. Harness replaces manual-host Skill calls with
resolved assets and makes flow designation explicit. The Griller grilling phase
requires the actual domain-modeling procedure and glossary/ADR format references.
Specification and tickets load their own procedures, not all roles or phases.
Later roles' dependency closures include their assigned procedures, message-first
handoff, verification/design or writing guidance as appropriate.

Harness-specific authority overrides upstream perpetual-session assumptions,
recursive/default delegation, reset/cleanup suggestions, embedded Implementer
review dispatch, generic permission policy and automatic Retro improvement work.
Verification follows the target repository's policy, including Harness's external
probes instead of permanent tests. No whole-spec time/spending cap is introduced.
Repository root/scoped instructions take precedence over adapted procedures.

## Explicit resolution

The workspace is the canonical Git worktree root (or canonical non-Git directory),
consistent with repository Griller identity. No home-directory skill search occurs.

Optional configuration files are:

- `<app_data_path>/instructions/config.json`: local defaults.
- `<repository>/.harness/instructions/config.json`: repository selections.

Example:

```json
{
  "format": 1,
  "root": "AGENTS.md",
  "overrides": [
    {"id": "procedure/grilling", "path": "grilling.json"}
  ]
}
```

An override path is relative to its configuration directory. Example override:

```json
{
  "format": 1,
  "version": "repository-grilling-2",
  "body": "Use the frontier interview and record resolved terms under our domain convention.",
  "requires": ["supplement/domain-modeling"]
}
```

Configuration entries are explicit choices, not same-name folder shadowing.
Repository entries override matching local IDs; local entries override the bundle.
Inspection lists source/precedence, including shadowed selections. Duplicate IDs
within a configuration, unknown IDs, unknown fields and incompatible formats
block. An override must preserve its bundled required dependencies; it may add
known dependencies. Missing, cyclic, invalid, empty, non-UTF-8 or oversized required
instructions block without falling back to the bundle or a generic prompt.
Unused assets are not loaded merely because their files exist.

With no explicit `root`, exactly one of root `AGENTS.md` or `CLAUDE.md` is selected.
Neither is a missing-required-guidance block; both are ambiguous until explicitly
selected in configuration. An explicitly selected root is required. Repository
and local root choices/overrides require explicit adoption before initial inference.
Approved handoff can capture the default CoS coordination closure; explicit overrides
are retained as unselected candidates until inspected/adopted before inference.
Pending routed Implementer intents inspect their `implementation` closure through
`/children guidance <intent>`. Adoption derives the phase from the exact prepared
intent and verifies its actual parent, approved work, current parent revision and
available hash. The compatibility-named Research guidance endpoint/record also
retains implementation snapshots; accepted intents cannot replace their closure.

The first default Griller run can capture the unambiguous root and bundled grilling
closure without a separate adoption command. A block occurs before creating a new
user/reply/run or contacting inference; repair and retry or inspect/adopt explicitly.

Scoped directories and reference files are explicit request paths, not a fuzzy
classifier or recursive documentation scan. For each selected directory, resolve
root-to-leaf `AGENTS.md`/`CLAUDE.md` along its ancestors. Scoped ambiguity blocks;
resolve it in the repository. Select referenced tracker/domain/wiki/standards
files when relevant, not the whole docs tree. Paths must be relative without
traversal; scoped directories and repository documents must resolve within the
workspace. Repository guidance remains higher priority than the role. Ordinary
source/tool reads remain observations; they are not silently substituted for
captured instruction authority. The role asks for explicit inclusion/adoption of
new required instruction context before work in that scope. This is guidance
selection, not tool-path confinement or a general policy sandbox.

## Snapshots, updates and recovery

A content-addressed snapshot records the selected phase, scopes, references, actual
instruction bodies, dependency closure, source paths, content versions and upstream
links. Its SHA-256 identity covers these values, not timestamps or a claim about
historical content. Conversation schema v4 introduced instruction persistence; schema six (and Research schema
seven) retain v5 deduplicated immutable snapshots,
the selected snapshot ID and each run's actual snapshot ID, in the existing private
bounded atomic/check-revision conversation file. Snapshot checksum/closure and run
references are validated on read/update. Corrupt records are rejected unchanged.

Selection is retained across subsequent runs on this durable Griller until explicit
adoption. Every new run captures the selected ID. Every generation, steering
continuation and explicit Resume uses that run's captured text. Edits/deletions,
bundle upgrades, restart and available-resolution failure cannot silently replace
it. No storage lock covers inference. Existing checkpoint-before-effect,
result-before-continuation, usage accounting, model/effort, Stop and Unknown gates
remain authoritative.

Inspection distinguishes **selected** (future runs), **current** (latest run's actual
snapshot, possibly an older selection), and **available** (resolution now). A new
phase or scope is an inspected candidate, not activation. Adoption requires the
inspected conversation revision and available SHA-256 to still match, and idle
work with no pending interruption. A changed file/revision asks for inspection
again. Adoption changes future runs only; prior run references/text remain intact.
The durable selected reference and immutable retained snapshots record adoption;
this is not a separate event/audit service.

Versions 1–5 still load, including existing v4 instruction snapshots and v5 provider provenance. Unassociated pilot conversations keep their historical
behavior without a fabricated role provenance. Historical Griller runs with no
snapshot remain explicitly unknown. An interrupted such run cannot Resume through
a newly invented snapshot: inspect history/effects, explicitly acknowledge the
interruption, then adopt instructions for a new run. Unknown effects remain Unknown.
New work can capture defaults; it does not retroactively annotate prior runs.

Limits: configuration 64 KiB/64 overrides, individual instruction file/body 256 KiB,
64 instructions/1 MiB body text per snapshot, 32 scope/reference paths each, and
128 retained snapshots, all additionally bounded by the existing 16 MiB conversation
record and pre-effect finishing reserve. Parsing/checksum work is bounded; JSON
parsing/record conversion/encoding use blocking workers, filesystem I/O uses Tokio.
There is no automatic compaction, deletion, inference or update polling.

## ACP and native inspection

Initialization advertises `harness/guidance: true`.

- `_harness/guidance/read`: `{cwd, sessionId?, phase?, scopes?, references?}`.
  With a session, its saved workspace and selected context are defaults. No
  session permits inspecting available instructions before creation/login.
  Returns `revision`, `selected`, `current`, `currentRun`, `historicalUnknown`,
  `available`, `blocked`, `overrides`, and `bundled`. Each snapshot includes
  `{id, phase, scopes, references, instructions}` and each instruction includes
  `{id, version, source, upstream, requires, body}`. No provider continuation or
  credentials are exposed.
- `_harness/guidance/adopt`: `{sessionId, revision, available, phase, scopes,
  references}`. Returns the ordinary safe `{conversation}` response. No inference,
  tool execution, file generation or new role dispatch occurs.
- Conversation metadata adds `selectedInstructions`; runs add nullable
  `instructions` and their captured phase as `task`. Unknown historical runs keep
  the previous task label, not invented instruction provenance.

In the stock launcher/Ink application:

- `/guidance` inspects; `s` selected, `c` latest run, `a` available, `b` bundled-only.
- `d` toggles exact instruction text; arrows/PgUp/PgDn scroll; `r` refreshes.
- Escape closes without stopping work or discarding chat drafts/scroll anchors.
  Inspection request waits are cancelled and late responses invalidated on
  dismissal, replacement, unmount or agent/target changes. Already admitted
  filesystem work may finish; cancellation does not claim to undo it. The last
  completed inspection is retained separately, so cancelling its replacement
  does not discard the candidate already inspected for adoption.
- Close inspection, then `/guidance adopt` explicitly adopts the inspected
  candidate. A first adoption can create the Griller using the selected model,
  without inference. Existing interrupted/active work cannot adopt. An
  uninterrupted adoption opens updated provenance, including when it creates the
  Griller. Completion cannot reopen a dismissed inspector or replace newer
  phase/reference selection, map/picker navigation or an edited draft. Dismissal
  does not cancel or roll back the independently durable adoption.
- `/guidance phase grilling|specification|tickets|coordination` inspects a phase candidate.
  Adoption is bound to the durable role; coordination cannot rename a Griller.
- `/guidance scope <relative-directory>` and `/guidance reference <relative-file>`
  add relevant context to the inspected candidate.
- `/guidance reset` inspects default grilling/root-only context; it is not adoption.

Inspection itself is allowed during an active run; other mutation commands remain
idle-only. The existing chat/map/title, Stop confirmation, editor cut, navigation
and draft behavior remain. Escape still never means Stop. The inherited Ink 20 ms
incomplete-prefix boundary remains documented and is not an unrestricted paste
pass. Native PTY byte observations are not human terminal visual signoff; scripted
provider fixtures are not live quality, account entitlement or paid-provider proof.
