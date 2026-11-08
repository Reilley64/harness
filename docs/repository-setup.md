# Repository Setup

## Detection and actor

Repository open uses `ReadSetup` before choosing a repository actor. Presence of
root `AGENTS.md` or `CLAUDE.md`, `docs/agents/issue-tracker.md`,
`docs/agents/domain.md` and `docs/agents/wiki.md` satisfies detection. Triage is
conditional: this adaptation does not bundle a triage executor; an existing root's
explicit `docs/agents/triage-labels.md` pointer selects its guidance, and then that
file is required. Setup can agree triage use and propose that pointer/mapping.
The detector does not validate guidance contents, wiki links, tracker capabilities
or credentials. Unreadable root content cannot select an otherwise unknown triage
pointer. Its instruction error surfaces on use, not as invented missing outputs.

Missing lazy glossary/ADR artifacts, absent tools and failed MCP/auth/remote access
never add missing outputs. There is no completion stamp, readiness registry,
health polling, fuzzy classification, migration or semantic-memory service.
A missing required guidance file opens Setup rather than rewriting complete files.

Setup is a separate durable System 0 actor, keyed by the canonical repository
identity and a Setup namespace. Griller's existing key/identity/history remain
unchanged. Both use the actual provider-bound factory, application-wide model
admission, ordinary tools, generation ledger, immutable instruction snapshots,
checkpoint coordinator, storage and Stop/Resume path. No inference or conversation
creation occurs just from opening the repository. Choose a model/authentication,
then send a message to start the detected actor. The accepted approved-work
handoff still creates a separate CoS through its own authority gate; Setup never
substitutes for that approval. [Native Decider selection and inspection](native-decider.md)
is available for approved CoS assignment intents; it does not launch a child.
Child execution remains unavailable.

Setup captures its setup-role/procedure closure. Bootstrap may lack a root or
have both roots: capture existing roots without silently choosing an owner, then
ask the intended target. An explicitly configured missing root is also a legitimate
Setup bootstrap omission. Griller keeps its strict required-root/dependency and
explicit-override adoption blocks. Overrides still require inspection/adoption
before initial inference; model admission precedes instruction loading/persistence.
Role-mismatched phase adoption is rejected.

## Conversation, preservation and wiki agreement

The adapted upstream procedure explores with actual tools and discusses one
decision at a time. Existing tracker/domain choices outrank bundled defaults;
a Git host does not select the tracker. Preserve root ownership/pointers, custom
state vocabularies and unrelated content. A new root is a human choice; when
one already exists, Setup cannot draft a duplicate. With both roots, agree which
owns navigation. A root-only instruction-configuration proposal can record that
choice while preserving format/overrides. Explicit configuration still requires
normal guidance adoption for future inference.

Discover/reuse historical wiki content before proposing a location. Agree its
entry point, organization and maintenance: dated historical claims, source links,
supersession rather than erased history. Put the durable pointer in
`docs/agents/wiki.md`, then append its navigation to the intended root for fresh
agents and the future CoS. A new index must be explicitly referenced in existing
or proposed wiki guidance with its exact relative path in backticks or a Markdown
link. Existing index/history is never replaced or moved by this workflow.

Setup's ordinary `write` tool captures a durable **draft**, not a repository write.
The draft contains the exact target, before/after text, canonical workspace and
both root contents plus repository instruction-selection configuration. Root
drafts append to the entire original byte prefix. Existing tracker/domain/triage/
wiki guidance and historical indexes are preserved. Only missing applicable
guidance, explicitly referenced missing Markdown wiki indexes and root navigation/
root-selection configuration are eligible. Setup's `edit` tool returns a failure
asking for a full preserved-content draft instead of bypassing inspection.

The human can discuss revisions, cancel a draft and ask for another. Apply missing
guidance/indexes before root navigation: changing a root/configuration invalidates
earlier captured proposals. A remote lookup failing later is operational, not an
instruction to restart onboarding.

## Trusted confirmation and effect boundary

Model prose, tool calls, tracker state and model completion claims never authorize
a stored draft. The native inspector shows exact before/after text and target.
Its explicit confirmation sends the proposal UUID and inspected conversation
revision through ACP. Another revision requires inspection again. This trusted
local client request authorizes **only that exact stored proposal**, not future
edits, external operations, installs, credentials, migration or broad provisioning.

Domain transitions own idle/interruption, role, proposal and transition eligibility
for confirmation and cancellation, along with duplicate-root, append-only and
eligible guidance/referenced-index value rules. Adapters observe files and parse
instruction configuration; the use case retains revision checks and effect ordering.
Confirmation holds the ordinary conversation lock, checkpoints Running before
the effect, then uses the ordinary canonical-path tool lock, expected-base atomic
replacement and bounded result/artifact machinery. Both roots/configuration and
target content are checked again. Relative paths cannot traverse, follow symlinks
or rebind to another canonical repository. Concurrent Harness confirmations are
serialized; a changed root/target fails without replacing editor content. An
external OS-user editor can still race advisory locks/path checks; this is not an
OS sandbox or an exactly-once filesystem guarantee.

The terminal result is saved before returning. Partial effects and Failed/Unknown
remain real; an effect whose result checkpoint is lost recovers Unknown. Recovery
does not retry it or manufacture consent. Reopen/load/Stop/kill do not approve
drafts. Explicit Resume continues interrupted model work with its captured model
and instructions, not uncertain writes. Inspect actual files and use
`/resume inspect` to acknowledge interruption for a later human inspection run;
Unknown remains Unknown and cannot be reconfirmed. No inference is used to
fabricate a handoff or decide whether an effect happened.

An admitted confirmation may finish after its panel closes or shutdown begins;
closing is not rollback. The service drains admitted writes even when no model
reply or other write is active, within the existing bounded shutdown window.
Acknowledgements reconcile only the same conversation at a non-older revision, never reopen a
dismissed panel or switch a newer role/navigation target. A cancelled draft is
Skipped and cannot later be confirmed.

This is a bounded repository-output workflow, not a per-role permission system.
Bash and future configured MCP remain trusted OS-user tools, not sandboxes.
Setup guidance requires exploration-only bash and separate human work/consent for
machine-wide installs, remotes, external resources or authentication. Arbitrary
trusted shell programs can still access/mutate OS-user resources; the draft gate
is not a security claim about them. The accepted base has no MCP executor; no
unaccepted MCP or OAuth candidate is imported or advertised here.

## ACP and native controls

Initialization adds `harness/setup: true`.

- `_harness/setup/read`: `{cwd, sessionId?}` returns `workspace`, `roots`,
  `missing`, `triage`, nullable `revision` and stored `proposals`. With a session,
  its saved workspace wins. Each proposal has the ordinary safe tool summary and
  `draft: {workspace, path, before, after, roots}`. This read is cancellable and
  performs no inference/mutation.
- `_harness/setup/update`: `{sessionId, revision, proposal, confirm}` confirms one
  exact draft or cancels it, returning `{conversation}`. It cannot edit proposals.
  There is no model tool exposing this human control.
- Agent read/start accept optional `role: "setup" | "griller"`; absent role uses
  detection. Explicit role access preserves each independent durable history.
  Unknown/future roles are rejected, not synthetic map nodes.
- Tool summaries add nullable draft metadata and terminal `draft` status. Draft
  means proposal persisted with no file effect, never Succeeded.

In `bin/harness`, `/setup` opens presence/draft inspection. Left/right chooses a
proposal, arrows/PgUp/PgDn scroll before/after, `r` refreshes. The full before/after
uses ASCII JSON strings: control characters, non-ASCII code units and directional
markers are escaped, not hidden or executed; escapes are displayed, not written.
The target is also an ASCII JSON string in both inspection and approval, including
quotes and escaped tabs, newlines and bidirectional markers. Confirmation retains
the exact original pathname without filtering or normalization.
`c` asks for exact
confirmation; `y` confirms, `n`/Escape dismisses without consent. `x` permanently
cancels that draft. Escape closes inspection and preserves the chat/editor.
Inspection is available during model work, but mutation remains backend idle-only.

`/setup chat` opens the real Setup actor; `/griller` opens the independent Griller.
Neither sends a message or inference. Automatic subsequent repository open bypasses
Setup once outputs are present; the open Setup conversation remains selected until
explicit navigation, not a background title takeover. Titles are
`harness - Setup`, `harness - Griller` and `harness - Agent map`. The map shows
only its actual current actor, not the later multi-agent map product.
F2 toggles chat/map; F3 opens the actual repository Griller from Setup, CoS or
the map, without inference. The conversation hook owns lookup, load, model
restoration and supersession; no screen-owned asynchronous lookup is added.
Independent per-actor editor drafts and in-session transcript anchors survive
navigation. Switching an active actor remains blocked rather than implicitly
stopping it. Existing editor, Stop, guidance and terminal cleanup controls remain.
Native PTY scripts establish bytes/effects, not human visual acceptance; Ink's
inherited 20 ms incomplete-prefix ambiguity remains, not arbitrary-prefix proof.

## Persistence and provenance

Conversation v6 adds independently owned optional draft records to tool calls and
permits the actual Setup role alongside the accepted v6 CoS assignment records.
Create/read/update/coordinator adapters retain their own representations of both
features, provider provenance, immutable instructions, run references, private
continuation and canonical identity. Typed `chat`/`decisions` capability arrays remain
authoritative; legacy booleans and missing metadata grant no capability. Existing
versions 1–5 still load with explicitly Unknown historical lineage.
Only v6 may associate Setup/drafts; missing draft metadata for Draft status,
invalid paths/bounds or mismatched role/workspace are rejected. There is no invented
legacy approval. Schema conversion, parsing and encoding stay in each owning adapter
on bounded blocking workers. Existing 16 MiB capacity/reservation gates still run
before effects; there is no compaction/deletion.

The procedure is Harness adaptation `harness-setup-2`, sourced from
`mattpocock/skills` 1.3.2 commit
`4ae2317ddb22b046e26455e4f9c9b3aaaf62212d`, engineering
`setup-matt-pocock-skills/SKILL.md` and its tracker/domain/triage seed references.
The bundled procedure is runtime-owned; personal/global skills are not dependencies.
Canonical product decisions: HARNESS-6 comment
`97c2fd8a-7073-49be-8eb0-796d23862f5d`, HARNESS-5 comment
`6613c412-4ca8-43ab-b076-c871b99a6bc3`; verification follows HARNESS-19 V02/V18
and HARNESS-11 comment `22ea98d6-091d-467a-b6f0-f9a921fb461d`.
