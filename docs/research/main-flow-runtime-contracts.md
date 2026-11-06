# Main Flow skills and runtime contracts

Researched **2026-11-06 (UTC)** for [Verify Main Flow skills and repository setup requirements (HARNESS-3)][question]. This is research, not an implementation spec or authorization to build. The parent must validate the findings and serialize the resolution; this report does not close the investigation or change the map.

## Answer in brief

1. **Main Flow is a composable, human-driven practice, not an executable orchestration engine.** Its primary route is grilling → spec → tickets → implementation → review → Retro, with small-change and prototype branches. Skills describe actions; the host must supply file access, executable tools, delegation, context handling and permissions. Most orchestration steps explicitly opt out of model invocation. Automatically wiring the proposed roles together therefore requires an explicit product contract, not simply loading the files. [S-router] [S-invocation] [S-specification] [S-integration]
2. **Setup configures each repository's tracker, not Plane specifically.** GitHub, GitLab and local Markdown have seed templates; “other” trackers use user-described workflows. Models should read and operate the repository's configuration. Documentation alone does not install a CLI/MCP client, authenticate it, grant permission, or make its operations durable. [S-setup] [D-setup] [S-mcp]
3. **The ten requested installed skills match the inspected upstream source byte-for-byte.** The important overrides here are Harness's repository instructions and the accepted map boundaries, not patched copies of these ten skills. Chief of Staff and software-factory remain in upstream's `in-progress` bucket and are absent from the promoted plugin manifest. [V] [S-manifest]
4. **Bounded runs require durable conversation continuity, not a perpetually live Chief of Staff.** Upstream Chief of Staff assumes a long session and wiki; the Main Flow router prefers retaining the grilling/spec/tickets conversation. The accepted product ends a run when it needs a human. These can coexist if the later run can retrieve the actual prior dialogue, artifacts and pending question, but the persistence/activation contract is not supplied by these skills. [S-chief] [S-router] [D-grill] [map]
5. **Review and Retro need deliberate adaptation.** `implement` requests review before its final commit while `code-review` reads committed `HEAD`; `implement-spec` leaves rework termination unspecified; standalone Retro proposes changes, whereas Chief of Staff takes authority to pursue them. Neither supplies the requested final-PR Retro footnote or human merge gate. [S-implement] [S-review] [S-implement-spec] [S-retro] [S-chief] [map]

## Accepted boundaries, not decisions made by this report

The accepted map requires a local, single-user, one-repository-at-a-time MVP in the existing TUI, allowing concurrent agents. Griller remains the main interface, **but the user can talk directly to any agent**. Setup establishes the repository's own tracker configuration; Plane is the tracker for developing Harness, not a product-wide dependency. Approval is required before implementation and before merge. Autonomous implementation/review loops must be bounded. Runs end for human input; later runs recover durable context. Approved specs, ticket progress, relationships, routing, reported usage and outstanding approvals survive restart. Unknown tool outcomes must not be blindly replayed. Retro findings belong in the final PR, with a footnote in its description. No execution while Harness is closed, remote runner provisioning, separate web UI, automatic deployment or unattended merge is required. Exact policy details remain open. [map]

The map's role intent is Setup; Griller using `grill-with-docs`, `to-spec`, `to-tickets`; Research callable by Griller and Chief of Staff; Chief of Staff coordinating Implementer, Reviewer and Retro through Decider; Implementer using `implement`; Reviewer using `code-review`; Retro using `retro` at spec completion. These role identities and Decider routing are **product intent**, not roles/dispatch machinery implemented by the Main Flow skills. [map] [S-router]

## Evidence, versions and limits

### Inspected snapshots

- Harness worktree started at **`6b4512d6ef7b0aa38783ff939bfd0f76234f41a4`**. Source/document observations below are pinned to that commit, not a claim that the factory has been run. [H-tools] [H-conversations]
- Upstream `mattpocock/skills` `main` resolved through GitHub's first-party API to **`4ae2317ddb22b046e26455e4f9c9b3aaaf62212d`**, commit timestamp **2026-10-31T14:23:30Z**. Both `package.json` and `.claude-plugin/plugin.json` say **1.3.2**. Every upstream source link below uses this commit rather than a moving branch. [S-commit] [S-package] [S-manifest]
- Installed files were read from **`/Users/reilley/.agents/skills/<name>/`** on the research date, including the ten requested `SKILL.md` files and their relevant local references. `/Users/reilley/.agents/.skill-lock.json` is installer schema version **3**, identifies `mattpocock/skills` as their source, and records per-folder hashes and update times. It does **not** record a source commit or establish one common installed release. Its observed SHA-256 was `76cfbd7042568de00a906608e79d2d2d4fed843b3d0cfa8d233dbc1ba974e3fc`. [V]
- Raw upstream files were fetched into temporary files outside the repository, not another Git checkout. Byte comparison against the installed copies found **all ten requested `SKILL.md` files identical**. The same comparison found the Setup templates, grilling, domain-modeling (and formats), TDD (and test/mocking references), writing-for-agents (and mechanics), wayfinder, ask-matt (and phase-boundaries reference) identical too. Software-factory's only observed `SKILL.md` difference is that its installed `description` is quoted; the body is identical. Its four service surveys are identical. [V]
- Agent Skills specification/integration documentation was fetched on **2026-11-06**; those URLs were not pinned to a source commit. MCP Tools was read at protocol revision **2025-11-25**. These establish available host-integration mechanisms, not behavior already implemented in Harness. [S-specification] [S-integration] [S-mcp]

### Installed identity ledger

`skillFolderHash` below is **installer metadata, not a verified Git commit**. The SHA-256 is of the actual installed `SKILL.md`, which matched the pinned upstream file. This separates provenance metadata from measured content. [V]

| Skill | Installed folder hash | Installed SKILL.md SHA-256 |
| --- | --- | --- |
| setup-matt-pocock-skills | `b644b684781fd5baae57f6be000a180a4d4448d7` | `b01e05bfe8c323171275c6ceec78b12f1bd4c62dfd26c2cfe8be806d3883f1ed` |
| grill-with-docs | `eedaf2562c83155115e9c649fa3ecaac2e10e81d` | `7de372c13488f1ee96cc11cd8907b56b6809cc93eef776eeddd37de6b6cbe3fe` |
| to-spec | `136b9651747cfccbfbb7c50a6b0dfb757e7547d2` | `43ad9cf318e5e7d3d1fa360253a37021796dc87a0c2e595ad262661a10f85088` |
| to-tickets | `2dee20ad722a31c04ededf9f2d83842473573045` | `56ec23c5b2782785d4e65bff9e315b6d3aeb4ef977c174a219e99489fada1995` |
| research | `2f8f36af48796faed5c687c75bb650071631a984` | `1f629e18ab899412683bc6a1c4efb4339999e27c2fdac328f26ee10a8c031d33` |
| chief-of-staff | `58e0a4122260ba45c150d8201dad2d6de4153d18` | `f4c1d6533edece6b275155911cb627154a6b13e2bc19934f77330f3afdd07987` |
| implement | `4804b466ab26c9b9d8d42917547db133d1f20a9f` | `483bdc4ccf4ac31d86ac7beeaa0d0db80313e71e120bb8c59b3196ae5decd53d` |
| implement-spec | `407c196f5c88af53c71efc8f31ee37c02dbb12e7` | `1993a473cec40ac85d081c112cdffdc63ec3267c2e412055ad09e013a2f623d1` |
| code-review | `a8ef561571acbb611de8ff19ba6da75b197c63a8` | `9c921a8d88138a41401d8897ca1bb852088ab1b7b605cdd04edc13e3e8aab6dd` |
| retro | `335c1862a43da9d629abafdb79ca0c0de118be08` | `33fbc28568e3146f20fd908b12998bc0ab30aa4f4c87288199a40793efa94696` |

Installed `updatedAt` spans 2026-10-16 through 2026-11-03 for these ten; differing timestamps do not contradict equal contents. In-progress Chief of Staff is installed through the universal skill path despite not appearing in the promoted 1.3.2 plugin list. Installing only that plugin cannot be assumed to provide it. [V] [S-manifest] [S-readme]

### Evidence hierarchy and observed documentation drift

For operational claims, this report follows the **actual `SKILL.md` and repository instructions**, using first-party human-facing docs to explain intent and reported limitations. Upstream's router explicitly requires reading the target skill before describing it or skipping a step. Its human docs describe themselves as secondary to the skill. [S-router] [D-router]

A concrete drift example: the Setup docs say it does not create triage labels, but the inspected Setup skill's Write step explicitly creates missing configured labels on GitHub/GitLab. The skill wins. Similarly, prose asserting that merging the integration tip guarantees a fast-forward is not a concurrency guarantee: the tip may advance again before landing. Neither these reports nor first-party descriptions of community incidents are independently reproduced runtime tests here. [D-setup] [S-setup] [D-implement-spec]

## Per-skill operational contract

All facts in this table describe the installed and matching pinned upstream instructions. “Human gate” means what those instructions request, **not** an enforced permission boundary. [V]

| Skill | Required reads/tools and delegation | Human gate | Expected artifact / stopping point |
| --- | --- | --- | --- |
| **Setup** | Explore remotes/config, root steering, prior `docs/agents/`, glossary/legacy context names, ADR locations, `.scratch/`, triage availability and monorepo signals. Read seed templates; write repository docs/steering; possibly initialize Git/add remote, validate `gh repo view` / `glab repo view`, create missing configured labels. | Present findings; one section/answer at a time; confirm tracker and overrides; show editable draft before writing; ask which steering file to create if neither exists. | Tracker, domain and conditional triage mapping docs; `## Agent skills` navigation in existing steering file. Not a skill installer, runtime client installer or deterministic validator. [S-setup] |
| **grill-with-docs** | Explicitly load **both** `grilling` and `domain-modeling`. Read source/glossary/ADRs; write resolved vocabulary and qualifying ADRs; grilling delegates factual exploration to fresh agents. | Ask the settled-prerequisite frontier with recommendations, wait for answers, and do not act until shared understanding is confirmed. Domain choices are the human's. | Inline glossary/ADR updates; much other reasoning remains only in conversation. No automatic complete decision ledger/spec. [S-grill] [S-grilling] [S-domain] [D-grill] |
| **to-spec** | Read conversation, code, glossary/ADRs; identify highest practical verification seams; publish spec to configured tracker, with ready-for-agent role. | Explicitly check proposed seams with user. “Do NOT interview” forbids reopening product discovery, not this seam check. | Spec with Problem Statement, Solution, User Stories, Implementation Decisions, Testing Decisions, Out of Scope, Further Notes. No implementation authority supplied by the ready role. [S-spec] [D-spec] |
| **to-tickets** | Fetch full referenced body **and comments**; optionally explore code/prefactoring. Build a dependency graph of fresh-context-sized vertical slices; publish one ticket/file per slice, native blocking edges where available, source parent relationship. Wide refactors have expand–contract exception. | Present numbered breakdown, dependencies and deliverables; iterate until user approves **before publishing**. | Approved dependency-linked tickets. Does not dispatch them or close/modify the parent. [S-tickets] |
| **research** | Caller starts a fresh background agent; agent investigates primary sources, follows claims to owners, writes one cited Markdown file in repo convention. | No intrinsic interview gate; supplied research scope controls the work. | One cited report plus returned path. No built-in budget/stopping rule or automatic later reuse. This task supplies leaf-agent and branch/publication rules. [S-research] [D-research] [S-wayfinder] [question] |
| **chief-of-staff** | Long-lived coordinator; all work in fresh background subagents; sparse context pointers. Read/update an interlinked historical wiki; consider optional schedules and environment improvement. Explicitly reads procedures for large work and sampled Retros. | If no shared wiki, agree a location before first write. Otherwise claims broad authority, including selecting and pursuing Retro candidates itself. | Wiki index/pages and delegated artifacts; ongoing strategic work, **not** a naturally bounded run. [S-chief] |
| **implement** | Fetch ticket and state title; ask if ambiguous. Load TDD where possible at pre-agreed seams; typecheck/single tests regularly, full suite once, invoke code-review. | No spec-approval or branch-creation gate within this skill. TDD requires previously confirmed seams. | Commit on **current** branch. No instruction to close ticket, check acceptance boxes, fix review findings, push or open PR. [S-implement] [S-tdd] [D-implement] |
| **implement-spec** | Read spec/task graph; fresh implementer per ticket in own branch/worktree, based on integration branch, TDD, merge integration tip before reporting. Fresh merger agent lands work; optional exploration agent writes shared notes **outside repo**. Final code-review and one fresh implementer for findings; cleanup worktrees. | No implementation/merge approval mechanism in skill. A PR is conditional on tracker close semantics or user request. | One integration branch; draft PR after first merge if requested/needed, then ready at end; otherwise tracker-specific resolution and branch report. [S-implement-spec] |
| **code-review** | Resolve fixed point and nonempty three-dot diff; collect commit list, standards sources and spec. Two fresh **parallel foreground** axes, Standards and Spec (skip latter only when genuinely unavailable). Preserve separate findings, citations and per-axis worst issue. | Ask for missing fixed point/spec source; missing spec may be explicitly acknowledged. | Two-axis report, not automated checks, fixes, merge, or certification. Repository standards override heuristic smell baseline. [S-review] |
| **retro** | Load writing-for-agents; read primary session transcript/logs and existing check/CI commands; inspect navigation, checks, standards, steering, tooling and information access. | Presents severity-ordered candidates to user; standalone procedure does not authorize applying them. | Environment-improvement candidates. No automatic persistent report, PR footnote, code change or merge step. [S-retro] [D-retro] |

### Dependencies are operational, not optional summaries

`grill-with-docs` itself contains only the instruction to load two skills. Without domain-modeling it may interview without preserving vocabulary; without grilling it has no interview discipline. TDD additionally refers to tests/mocking guidance and codebase-design when the interface is in question. Retro depends on writing-for-agents; domain-modeling references its glossary and ADR formats. Loading just role names or one-line descriptions cannot reproduce these contracts. [S-grill] [S-tdd] [S-domain] [S-retro] [S-writing] [D-grill]

The literal tool name **“Skill” is not a cross-runtime requirement**. The Agent Skills integration guide permits ordinary file-read activation or a dedicated activation tool, plus host-handled explicit user invocation. A compatible runtime must deliver the real instructions and resolve references against their actual skill directory; printing “used TDD” does not establish that it loaded them. [S-integration]

There is an upstream packaging wrinkle: Chief of Staff lives at `skills/in-progress/chief-of-staff/`, but uses `../retro/`, `../to-spec/`, `../to-tickets/`, `../implement-spec/` references. In the inspected tree those targets live under `skills/engineering/`, so those sibling links do not resolve in the source layout. They **do** resolve in this machine's flattened `~/.agents/skills/` installation. A future bundled distribution must verify dependency resolution rather than assume either layout. [S-chief] [S-tree] [V]

## Setup detection and tracker execution

### What “already set up” means in the actual skill

Setup is prompt-driven, not a script with a completion marker or a versioned setup schema. Its Explore step reads prior output and surrounding conventions; its Write step updates an existing Agent skills block without duplicating it or overwriting surrounding edits. It edits `CLAUDE.md` if present, otherwise `AGENTS.md`; if neither exists it asks which to create. Triage output is conditional on triage being installed. Missing glossary/ADRs are normal because domain modeling creates them lazily. Therefore none of these alone establishes absence: no glossary, no ADR directory, no triage mapping when triage is absent, or no dedicated setup stamp. [S-setup] [S-domain-template]

For this repository, the observed `AGENTS.md` already points to tracker/triage/domain docs; those docs configure Plane, state-based triage and Plane-page ADRs. Git's local `plane.workspace=reilley` and `plane.project=64f3c6d2-bca8-48e2-9984-592b36890568` supply repository-local target identifiers. That is evidence of **existing configuration**, not proof that the product has executable Plane tools. No Setup rewrite was performed. [H-agents] [H-tracker] [H-triage] [H-domain] [V]

**Research-derived distinction for the product's detection contract:** evaluate these separately, without silently choosing policy:

1. **Skill availability:** are the required instruction files/dependencies installed, parseable, trusted and resolvable, including user-only skills hidden from the model catalog?
2. **Repository configuration:** are the appropriate steering pointers and tracker/domain/conditional triage documents present and internally usable, including custom layouts?
3. **Executable capability:** can the selected tool route target the intended repo/project and perform the required reads/writes with current identity/permissions?
4. **Compatibility/drift:** do current instructions require operations absent from old config or tools? Do repository overrides conflict with the selected runtime?

These are distinct because skill discovery/loading, Setup's prose configuration and actual tool invocation are separate mechanisms in the primary sources. The exact automatic classification, user prompt and repair/reverification rules are **unresolved product choices**. A missing capability must not be mistaken for permission to replace the configured tracker with Plane/GitHub. [S-setup] [S-integration] [S-mcp] [map]

Upstream explicitly declines a separate Setup verify/check mode: the intended interface is invoking Setup with a natural-language verification-only request. Setup's closing message says re-run to switch tracker/start over; changelog entries also recommend rerunning after seed-template changes. No upstream deterministic migration/version gate resolves that tension for Harness. [S-verify] [S-setup] [S-changelog]

### What executable tracker access needs

The templates distinguish GitHub (`gh`), GitLab (`glab`), local Markdown (read/write files), and another programmatic workflow supplied by the user. A CLI-based route can use process tools; an MCP route needs actual server discovery, exposed schemas, invocation and error/results handling. A prose instruction “use Plane MCP” supplies none of those by itself. MCP explicitly separates listing tools from calling them and distinguishes execution failures from protocol errors. [S-gh] [S-glab] [S-local] [S-mcp]

For this flow, the **capability inventory**, not a universal tracker adapter API, is:

- Fetch referenced spec/ticket bodies and comments completely; list scoped tickets and resolve identity/state/label identifiers.
- Publish specs and tickets; preserve canonical identifiers/URLs; set the repository's ready role; establish parent and blocking links (native where available; documented text/file fallback otherwise).
- Read blocker/completion state; claim/update progress where repository conventions require it; append findings/questions/resolutions; close work only by the configured convention and authorized lifecycle.
- For repositories that put ADRs in the tracker, read/write/link those records too.
- Separately access the Git host for branches, PR descriptions/reviews/checks and merge actions as authorized. The tracker and Git host need not be the same service. [S-spec] [S-tickets] [S-implement-spec] [S-gh] [S-glab] [S-local] [H-domain] [map]

A declared CLI version/flag in a seed template is **not** a local availability test. For example, GitHub's template gives newer CLI operations plus older API fallbacks; GitLab's template notes paid-tier native blocking links with a text fallback. This report did not test those vendor features or buy access. Supported tracker operations, fallback acceptance and plan constraints still need per-selected-repository verification. [S-gh] [S-glab]

The existing Harness process tool also deliberately filters environment variables, so an inherited developer shell's authenticated setup cannot be assumed to work identically in product subprocesses. The needed executable, target, identity, credential delivery, network reachability and read/write scope must actually be available in the product tool path. No credential was requested for this research. [H-security] [H-tools]

### Harness repository conventions are an override, not the product schema

At the inspected Harness commit:

- Issues/specs use **Plane**, while source is on GitHub. Spec/ticket work uses native parent/dependency fields; the five triage roles are **states**, not labels. Specs carry `spec`; wayfinding uses `wayfinder:*` labels. “Ready” means Ready for agent, unassigned and without an open blocker. [H-tracker] [H-triage]
- Descriptions/comments are HTML with no newlines adjoining block tags, avoiding Plane editor empty paragraphs. UUID fields must use resolved identifiers. Ticket bodies omit redundant Parent/Blocked by sections when fields already own that information. [H-tracker]
- Specs have additional section/citation conventions; ADRs live as child Plane pages under Architecture decisions, not `docs/adr/`. Behavioral verification uses **temporary external probes**, with no permanent tests/test-only dependencies. [H-tracker] [H-domain] [H-standards]
- For this research, the explicit task overrides the ordinary wayfinder resolve sequence: leave the existing claim/state alone, push a throwaway research branch, add only a pointer comment, do not close the ticket or edit the map. [question] [map]

These facts demonstrate why Setup output must be read as repository policy rather than overwritten with upstream defaults. They do not authorize a Plane-only product, nor impose Harness's temporary-probe policy on other repositories. [S-setup] [map]

## Runtime, delegation and durability implications

### Baseline product support is not inherited from this research host

Harness at `6b4512d` has nine fixed tools (`read`, `bash`, `edit`, `write`, `ls`, `find`, `grep`, `ast_search`, `web_search`). Its tool documentation explicitly excludes MCP, plugins, subagents, persistent shells and managed background jobs. Web search does not fetch result pages. Tools execute automatically with OS-user permissions, without approval cards; a workspace is not a security sandbox. `AgentRole` enumerates eight names, but the map/source baseline does not supply the proposed factory paths. The MCP/delegation tools available to this research session are **not** proof they exist in the Harness product. [H-tools] [H-tool-enum] [H-role-enum] [map]

A process tool could run installed Git/CLI/HTTP commands, but that is an executable route to be verified and permissioned, not an existing tracker integration. In particular, unrestricted shell writes could bypass a nominal approval UI unless enforcement covers the real mutation path. This is a consequence of the documented unsandboxed automatic tool boundary, not an assertion that one particular sandbox must be built. [H-tools] [H-security] [map]

### Delegation requirements and limits

Upstream uses several distinct patterns:

- Research and factual exploration: fresh background agents.
- Chief of Staff: all work in fresh background agents; sparse context pointers; optional schedules “harness-permitting.”
- `implement-spec`: fresh per-ticket branch/worktree implementers, background where possible, plus merger and optional exploration agents.
- `code-review`: two fresh, concurrent **foreground** reviewers, reporting back before aggregation. Independent of authoring context; a missing spec cannot count as a Spec pass. [S-research] [S-grilling] [S-chief] [S-implement-spec] [S-review]

These imply support for fresh contexts, tool access per child, observable completion/results and appropriate shared artifact visibility. They do not specify maximum concurrency, permitted nesting, timeout/retry budgets, spending limits or who may cancel/replace a run. Upstream explicitly places recursion guards in the **host**, not individual skill prose; its research/review docs describe recursion incidents but are not proof of safe limits. This research agent follows the explicit leaf-agent instruction and starts no children. [S-recursion] [D-research] [D-review] [question]

A worktree isolates an index/working tree, not secrets or all refs, and does not guarantee conflict-free integration or complete verification fixtures. The first-party implement docs flag shared `refs/stash`; implement-spec docs flag ignored fixtures/databases/credentials absent from worktrees. Its suggestion to use a main checkout for such tests is advice, **not** accepted permission to abandon the product's isolation boundary. Verification environment provisioning and serialized landing remain open. [D-implement] [D-implement-spec] [map]

### Durable artifacts the instructions actually rely on

| Durable material | Source expectation | Consequence for the accepted product (inference, not a chosen storage design) |
| --- | --- | --- |
| Repository setup | Committed `docs/agents/*.md` and steering pointers; actual Git target/config may live outside tracked files. | Preserve/configure target resolution across worktrees and restart; do not equate cloned prose with available auth. [S-setup] [H-tracker] |
| Grilling rationale | Glossary and qualifying ADRs are written inline; remaining decisions stay in the dialogue. | Preserve retrievable prior answers and pending questions, not only glossary/ADR files. [D-grill] [S-router] [map] |
| Approved spec and tickets | Tracker/local files; dependencies, parent IDs, acceptance criteria. | Record the revision/snapshot approval refers to and reconcile external updates; exact invalidation rules require human decision. [S-spec] [S-tickets] [map] |
| Research | Cited Markdown report; wayfinder adds throwaway pushed branch and ticket pointer. | Artifact must remain reachable by later runs; immutable commit links are useful, but deletion/retention policy is not supplied. [S-research] [S-wayfinder] |
| Exploration notes | `implement-spec` explicitly permits shared files outside repo. | A temporary runner-local path alone fails later-run durability; publication/copying to durable orchestrator artifacts needs a contract. [S-implement-spec] [S-factory] [map] |
| Chief of Staff knowledge | Existing shared wiki or agreed location; index plus linked pages; dated claims and commit-pinned code history; ingest/query/lint. | Runtime progress state is not the same thing as a curated wiki. Whether/how to add this memory surface is open, not authority to create a new subsystem. [S-chief] [map] |
| Code/review evidence | Integration and child commits, review fixed point/diff, separate axis findings; checks and results. | Later runs need the reviewed revision and findings; tracker “closed” and “landed in integration” may differ until PR merge. [S-review] [S-implement-spec] [D-implement-spec] |
| Retro input/output | Primary session logs; severity-ordered environment candidates. | Give Retro access to the relevant spec's runs, not just its own fresh context; persist findings and link final PR footnote. Selection/application authority remains open. [S-retro] [map] |
| Operational state | Factory orchestrator records queued/in-flight/finished runs; accepted map adds relationships, routing, usage and approvals. | Persist starts/results before continuation; preserve unknown outcomes rather than replaying mutations. Existing conversation durability is a useful constraint, not complete factory state. [S-factory] [H-conversations] [map] |

## Conflicts requiring explicit adaptation

These are not reasons to discard the skills, and not decisions to override them silently.

| Tension | Source fact | Unresolved adaptation / accepted constraint |
| --- | --- | --- |
| **Automatic roles vs user-only invocation** | Setup, grill-with-docs, to-spec, to-tickets, Chief of Staff, implement, implement-spec and Retro set `disable-model-invocation: true`; research/code-review do not. Upstream invocation rules prohibit a user-only skill invoking another user-only skill, yet in-progress Chief of Staff explicitly reads several such procedures. | The product must explicitly define how human-approved role execution loads these procedures. Do not infer automatic activation authority from role names, or label hidden skills “not installed.” The generic Agent Skills spec does not itself define this host extension. [S-invocation] [S-chief] [S-integration] |
| **Live interview/long Chief of Staff vs bounded runs** | Grilling waits for answers; router prefers one unbroken grilling/spec/tickets context; Chief of Staff describes a single long-running session. Factory says a run never waits for a human. | Accepted: persist question/context, end run, start another on human answer. A durable conversation may span runs without leaving inference open. Exact steering/cancellation and continuation semantics remain open. [S-grilling] [S-router] [S-chief] [S-factory] [map] |
| **Tracker as sole human interface vs direct agent conversation** | Factory calls tracker the human interface and places later human-trigger changes in tracker/Git host. | Accepted product also has TUI conversation with any agent, Griller primary. Define which direct replies change durable work/approval state and how conflicting instructions are resolved; do not forbid direct agent access. [S-factory] [map] |
| **Ready label vs implementation approval** | to-spec/to-tickets mark artifacts ready; to-spec docs warn a polling agent may implement the parent spec rather than slices. | Accepted human approval before implementation is separate from triage readiness/publication. Which revision/ticket set it covers and when it expires remain open. [S-spec] [S-tickets] [D-spec] [map] |
| **Review-before-commit visibility** | implement invokes review before its final commit; review uses `git diff <fixed-point>...HEAD`, excluding working/staged changes. | Need an explicit reviewed-commit/snapshot contract (e.g. an interim commit) and fixed point before dispatch. The skill sequence alone can review an empty/old diff. No product choice made here. [S-implement] [S-review] [D-review] |
| **Ticket resolution vs dependency frontier** | implement does not close work; implement-spec aims to resolve all tickets. A tracker may close only after PR merge, so externally open blockers can already be integrated. | Need durable distinction between implementation/landing progress and tracker closure, and owner for tracker updates. Do not fake closure to unlock work or implement the entire graph again on restart. [S-implement] [S-implement-spec] [D-implement-spec] |
| **Review/rework with no convergence guarantee** | Review aggregates findings, not fixes; implement-spec requests one fresh fixer but no clear after-fix stop; docs warn broad review can keep discovering smells. | Accepted bounded autonomous loops; exact cap, severity policy, focused recheck, escalation and budget rules need human approval. Scope whole-spec review only after all tickets land, not against incomplete work. [S-review] [S-implement-spec] [D-review] [D-implement-spec] [map] |
| **Integration merge vs final human merge** | implement-spec merges ticket branches autonomously into integration. Neither skill provides a protected final-merge operation. | Accepted direction permits bounded coordination after approval, but “merge” must distinguish integration landing from final protected-target merge. Exact approvals/permissions remain open. [S-implement-spec] [map] |
| **Retro proposing vs Chief of Staff pursuing** | Standalone Retro presents candidates; Chief of Staff overrides its last step and selects/pursues them itself, sampling roughly one in five sessions. | Accepted Retro at spec completion plus final PR findings/footnote. Do not adopt Chief of Staff's self-authorization by accident. Whether improvements join the same PR, become tickets, or require fresh approval/review/checks is a human decision. [S-retro] [S-chief] [map] |
| **Optional PR vs required MVP final PR** | implement-spec can finish offline on a branch, without PR. Retro contains no PR operation. | The accepted MVP asks for a final PR, so the product workflow must explicitly request one and publish Retro evidence; Git-host support is still undecided. Local tracker support does not imply a no-PR MVP exception. [S-implement-spec] [S-retro] [map] |
| **Disposable runner memory vs local persistence** | Factory treats runner state as disposable; only tracker/Git host/orchestrator carry memory. | Harness runs locally, but durability cannot depend on a child context or scratch path. No need to provision remote machines; local artifact ownership/retention remains to be defined. [S-factory] [map] |
| **TDD defaults vs repository verification policy** | TDD expects red-before-green at confirmed public seams; Harness forbids permanent tests/test-only dependencies. Retro says standards are for reviewers, whereas Harness requires reading standards before edits. | Repository instructions govern Harness work. Reconcile the technique through approved temporary probes rather than adding permanent tests or skipping verification; other repositories follow their own configured rules. [S-tdd] [S-retro] [H-standards] [H-agents] |

## Skill trust, versioning and setup requirements

Primary sources establish a directory format and progressive disclosure, not a safe execution sandbox or universally enforced version lock. Agent Skills supports optional metadata/compatibility and experimental `allowed-tools`; it does not make metadata an authority grant. Its integration guide recommends trust checks before loading project skills, deterministic collision precedence, diagnostics for parse failures and exclusion of user-only skills from the model catalog. These are integration guidance, not a selected Harness policy. [S-specification] [S-integration]

The following are **research-derived requirements to settle**, not a proposed implementation:

- **Enumerate availability separately from model discovery.** User-only skills must remain detectable for Setup/explicit invocation, even though absent from the model catalog. Validate dependencies and references, not just a top-level folder. [S-integration] [D-router] [S-grill]
- **Record the actual activated content and origin.** Name, resolved path/scope, source identity, revision/hash and explicit local override are materially different. A run must not retrospectively claim an upstream version solely because its installer lock names that repo. This report's byte comparisons establish identity only for inspected content. [V] [S-specification]
- **Handle drift explicitly.** Upstream supports plugin auto-updates and editable copies; its docs note direct local skill changes may be overwritten. Choose when updates take effect for in-flight/approved work, and how repository Setup drift is surfaced. No silently changing approved instructions mid-run. [S-readme] [D-router] [S-changelog] [map]
- **Choose trust/precedence and mutation authority.** A repository can supply steering, skill instructions, scripts and third-party tool outputs; those must not implicitly expand permissions or approve implementation/merge. Host-side control matters particularly when unrestricted shell/file access can bypass a tool-specific policy. Exact allowlisting, filesystem boundaries and user consent remain open. [S-integration] [S-mcp] [H-security] [map]
- **Separate format validation from behavior/access validation.** A parseable skill, a visible tracker doc, an available executable and an authorized successful remote operation prove different things. Upstream's changelog records invalid unquoted YAML causing undiscoverable skills; installed software-factory has a local quotation repair. Parsing and tool-access failures must be visible, not prompt-driven claims of success. [S-changelog] [S-integration] [V]
- **Control delegation centrally.** Fresh context does not mean leaf permissions; disabling nested delegation for a research/review leaf is a host decision. The source explicitly assigns recursion protection to the host. Exact depth/concurrency/cost caps remain unspecified. [S-recursion]

## Unresolved facts and product decisions

### Facts not established by this investigation

- No end-to-end factory execution, product skill activation, tracker connector, delegation or approval path was run. This was source research, plus actual read access to HARNESS-1/HARNESS-3 and their empty comment lists through this session's Plane tools. That does not validate product runtime access. [V] [H-tools]
- No complete supported-tracker/vendor feature matrix, chosen Git-host API or live `gh`/`glab` permission probe was performed. Seed-template examples are evidence of intended operations, not verified current vendor/installed CLI support. No paid service call or credential request was made. [V] [S-gh] [S-glab]
- There is no verified common installed Git commit in the lock file. The stated ten-file equivalence to pinned upstream is measured; installation provenance beyond that remains unknown. [V]
- Direct Claude Code documentation fetching was blocked with HTTP 403; no claim here relies on that fetch. Invocation facts come from Matt Pocock's pinned source and Agent Skills integration docs, not an unverified claim about all hosts. [V] [S-invocation] [S-integration]

### Human decisions still needed

1. Which executable tracker paths/capabilities are supported initially, how Setup verifies existing access without unwanted writes, and what constitutes “configured but blocked” versus “needs Setup”? Repository-driven choice is settled; a Plane-only restriction is not acceptable. [map]
2. How explicit user-only skill semantics map onto automatic roles after approval; skill source/trust precedence, pinning/update behavior, and setup drift handling. [S-invocation] [S-integration]
3. Exact approval scope/revision and invalidation rules, including direct conversation with another agent, steering during work, changes to spec/tickets/config and final PR updates. [map]
4. Bounds for concurrency, nested delegation, retries, rework, spend/time and escalation; integration landing authority versus final human merge. [map] [S-recursion]
5. Ownership and retention of context, logs, research/exploration artifacts and optional Chief of Staff wiki; how durable pointers are kept reachable without turning stale research into product policy. [S-chief] [S-research] [S-factory] [map]
6. Whether Retro only publishes findings or can apply selected improvements, how approval is obtained, and which resulting revision gets reviewed/checked before the final PR and human merge approval. Exact footnote/report location is not supplied upstream. [S-retro] [S-chief] [map]

These questions should return to Griller/the human with this evidence, not be answered by a research agent. The report intentionally does not select a permission model, tracker adapter architecture, memory subsystem, loop limits or Retro mutation policy.

## Verification and publication scope

This branch changes only this report. Temporary checks verified that every citation label resolves, all ten recorded installed `SKILL.md` SHA-256 values match the files, and the requested files match their pinned upstream counterparts. `mise run lint:secrets` passed with no leaks. `git diff --check` passed. Product build/test/runtime probes were not run for this documentation-only investigation. The branch/commit publication pointer is recorded in the research ticket comment; no PR or merge is part of this task.

## Sources

All upstream skill links below are first-party source at the inspected commit. Human-facing docs are explicitly secondary explanations when they describe other skills; operative claims were checked against the skill itself.

[question]: https://app.plane.so/reilley/browse/HARNESS-3/
[map]: https://app.plane.so/reilley/browse/HARNESS-1/
[V]: #evidence-versions-and-limits
[S-commit]: https://api.github.com/repos/mattpocock/skills/commits/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d
[S-tree]: https://github.com/mattpocock/skills/tree/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills
[S-package]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/package.json
[S-manifest]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/.claude-plugin/plugin.json
[S-readme]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/README.md
[S-changelog]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/CHANGELOG.md
[S-router]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/ask-matt/SKILL.md
[S-invocation]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/.agents/invocation.md
[S-setup]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/SKILL.md
[S-gh]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/issue-tracker-github.md
[S-glab]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/issue-tracker-gitlab.md
[S-local]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/issue-tracker-local.md
[S-domain-template]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/setup-matt-pocock-skills/domain.md
[S-grill]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/grill-with-docs/SKILL.md
[S-grilling]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/grilling/SKILL.md
[S-domain]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/domain-modeling/SKILL.md
[S-spec]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/to-spec/SKILL.md
[S-tickets]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/to-tickets/SKILL.md
[S-research]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/research/SKILL.md
[S-chief]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/in-progress/chief-of-staff/SKILL.md
[S-implement]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement/SKILL.md
[S-implement-spec]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement-spec/SKILL.md
[S-review]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/code-review/SKILL.md
[S-retro]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/retro/SKILL.md
[S-tdd]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/tdd/SKILL.md
[S-writing]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/writing-for-agents/SKILL.md
[S-wayfinder]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/wayfinder/SKILL.md
[S-factory]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/in-progress/software-factory/SKILL.md
[S-verify]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/.out-of-scope/setup-skill-verify-mode.md
[S-recursion]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/.out-of-scope/subagent-recursion.md
[D-router]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/ask-matt.md
[D-setup]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/setup-matt-pocock-skills.md
[D-grill]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/grill-with-docs.md
[D-spec]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/to-spec.md
[D-research]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/research.md
[D-implement]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/implement.md
[D-implement-spec]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/implement-spec.md
[D-review]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/code-review.md
[D-retro]: https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/docs/engineering/retro.md
[S-specification]: https://agentskills.io/specification
[S-integration]: https://agentskills.io/integrate-skills
[S-mcp]: https://modelcontextprotocol.io/specification/2025-11-25/server/tools
[H-agents]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/AGENTS.md
[H-standards]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/CODING_STANDARDS.md
[H-tracker]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/agents/issue-tracker.md
[H-triage]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/agents/triage-labels.md
[H-domain]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/agents/domain.md
[H-tools]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/tool-calls.md
[H-conversations]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/conversations.md
[H-security]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/docs/security.md
[H-tool-enum]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/domain/src/tool.rs
[H-role-enum]: https://github.com/Reilley64/harness/blob/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4/domain/src/agent_role.rs
