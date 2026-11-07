# Configured tracker access: smallest viable runtime paths

Verified 2026-11-06. Planning research for [HARNESS-13][question], feeding
[HARNESS-6][setup] under [HARNESS-1][map]; not an implementation or product decision.

## Bottom line

- **Bash already supplies a general execution route, not authenticated tracker access.**
  An installed, configured CLI or a user-managed script using an official SDK/API can
  suffice without product tracker adapters. This repository explicitly says **use Plane
  MCP**: raw API/SDK substitution needs an agreed instruction change; MCP via bash
  needs an external MCP client. Reading instructions installs neither route. [H][prior]
- **Minimal native MCP is a generic tools client**, not a Plane/GitHub integration:
  approved server configuration, initialization, tool discovery, schema-aware calls,
  bounded results, credential delivery, cancellation and durable uncertain outcomes.
  Stdio plus a provisioned server avoids native remote OAuth initially. [M1][M2][M3][M4][M5]
- A concrete **candidate**, not a chosen scope, is Plane's official stdio server for
  tracker operations plus `gh`/Git for GitHub PRs. Native Streamable HTTP with Plane's
  documented PAT endpoint is another bounded option; native OAuth costs more. [P1][G1][G3]
- Do not promise “all trackers supported,” verified write permission, atomic claims,
  exactly-once mutations, or enforced role permissions. Missing required capabilities
  must block readiness or prompt for an explicitly accepted fallback—not change trackers.

## Evidence and version boundaries

| Evidence | Inspected version / result |
| --- | --- |
| Harness source | `6b4512d6ef7b0aa38783ff939bfd0f76234f41a4`; process/security/tool docs and source inspected, no product execution. [H] |
| Existing research | Main Flow runtime contracts, `7557855810075aab0436b7c24e2b4520fead3532`, especially Setup/access distinctions. [prior] |
| MCP standard | Version **2025-11-25**, source `c9b51b3a02f3851f9821d5191718652bab97adac`; not every deployed server's negotiated version. [M1][M2][M3][M4][M5] |
| Plane MCP | Official source `5c3467adb36fe15a2ef8f9df6fb9f5dc24b6d881`; package metadata says **0.3.5**, pins `plane-sdk==0.3.1`. Not installed/run here; source snapshot is not proof of hosted deployment identity. [P1][P2] |
| Plane SDK | Official **v0.3.1**, `721061d6f0d0047e431dd5fed66920d31296ba2b`, matching that dependency; source, not just README examples, checked. [P6][P7] |
| GitHub CLI/API | Installed `gh 2.102.0 (2026-09-30)`; local help/tagged source confirm relationship flags. Official live manuals/REST references read, API reference selector `2022-11-28`. [G1][G3][G4][G5][G6][G7] |

Actual nonmutating observations: this **development session's** Plane tools read full
question/map/setup bodies, research comments, states and native dependencies:
HARNESS-13 blocks HARNESS-6 and has no blockers. Local Git returned workspace
`reilley` and project `64f3c6d2-bca8-48e2-9984-592b36890568`, matching the question.
`gh api --method GET repos/Reilley64/harness` returned the intended public repository,
default `master`, and `permissions.push=true`; only selected nonsecret metadata was
displayed. These are host-access observations, **not product preflight**, token-scope
proof or successful writes. No `plane` executable was found on PATH—not proof that
no official CLI exists anywhere.

No personal credential/configuration files or credential values were inspected;
no secret requests, server launches, mutation probes, workflows or paid calls.
Report/pointer publication is separately authorized. Plane's developer docs returned
**HTTP 403** for introduction, work-items overview and `llms.txt`; API claims below
rely on official SDK/server source, not unseen docs or examples alone.

## What the current product can actually do

`bash` starts `/bin/bash --noprofile --norc -c` in the saved workspace: no stdin or
persistent shell, default 120-second/max 600-second timeout. It clears the environment,
restoring an allowlist including `PATH`, `HOME`, XDG configuration/cache/data paths
and toolchain/locale settings—not `GH_TOKEN`, `GITHUB_TOKEN`, `GH_HOST`, `GH_REPO`,
`GH_CONFIG_DIR`, `PLANE_API_KEY`, `PLANE_WORKSPACE_SLUG` or `SSH_AUTH_SOCK`.
Shell rc setup is not loaded. Commands can set variables explicitly, but secret
literals in arguments enter recorded/model-visible tool content. [H]

`gh` may use its credential store/configuration via HOME/XDG, or fail if it needs
filtered variables/an unavailable keychain. Git push uses separate Git credentials/
SSH configuration; API reads do not verify it. A trusted wrapper could retrieve
credentials without printing them: a **route to verify**, not existing product proof.
Do not broaden inheritance wholesale or put secrets in prompts/docs/commands. [H][G1][G2]

The product has nine fixed tools, **no MCP client**, server/plugin discovery or
approval cards. Its helper closes stdin/drains stdout as text: not reusable unchanged
for bidirectional JSON-RPC. Tools are unsandboxed OS-user operations; HOME credential
access remains possible. MCP-only restrictions cannot constrain unrestricted bash/
file access. Minimal global approval/effect boundaries remain explicit; no full role
framework is required or supplied here. [H][M1]

## Setup/access information needed regardless of route

**Design proposal**, not an existing schema or MCP-defined configuration format:
1. Load the repository's steering pointers and tracker instructions; resolve the
   configured tracker instance, workspace/project, state vocabulary, identity and
   git-host repository independently. Preserve this repo's Plane choice, native
   blockers, HTML conventions and state-based triage. Never infer tracker from origin. [H]
2. Record the selected executable/server identity and version, invocation contract,
   working directory, target resolution, timeout/output bounds and nonsecret auth
   handle. CLI: explicit host/repo arguments where supported; MCP: server ID plus
   stdio executable/argv or HTTPS endpoint and approved header/credential references.
3. Separate repository-owned **nonsecret target/instruction data** from user-approved
   executable/endpoint trust and credential provisioning. MCP does not standardize a
   universal `mcpServers` file. Do not auto-launch commands from an untrusted repo or
   silently import this development harness's private configuration. [M1][P1]
4. Inventory required operations below, retaining evidence per operation and target:
   unavailable / discovered / read-verified / authorized-write-verified / unknown.
   Ask for repair or an accepted fallback when an instruction cannot be executed.
   Reverify after target, executable/schema, identity/auth or instruction changes.

## Official Plane and GitHub paths and capability limits

**Plane paths (documented/source-implemented; not live-tested here):** [P1][P2][P3]

- Local: `uvx plane-mcp-server stdio` (Python 3.10+), with
  `PLANE_API_KEY` and `PLANE_WORKSPACE_SLUG`; optional `PLANE_BASE_URL` for self-hosting.
  Provision/pin the server before a run rather than implicitly fetching latest code.
- Hosted OAuth: `https://mcp.plane.so/http/mcp`; hosted PAT:
  `https://mcp.plane.so/http/api-key/mcp`, using `Authorization: Bearer …` and
  `X-Workspace-slug`. PAT transport is Plane's documented alternative, **not** proof
  of implementing the standard's OAuth flow. Old `/sse` is deprecated.
- README offers `mcp-remote` as an external bridge (Node 22+). Its example does not
  verify a pinned bridge's header/config semantics or installation; check separately.
- Official Python `PlaneClient` supports an API key **or** access token and builds
  `/api/v1` requests; the key becomes `X-Api-Key`. A repo-managed script using this
  client is a viable process alternative if explicitly accepted. No product adapter
  is necessary, but runtime/package/auth provisioning is still necessary. [P6][P7]

| Required operation | Plane official MCP/SDK surface | GitHub official CLI/API surface |
| --- | --- | --- |
| Read full spec/tickets/comments, discover targets | `workitem` list/retrieve/retrieve_by_identifier/search; `workitem_comment list`; project/member/state/label tools. Follow cursors and fetch complete required content. [P4][P5] | `gh issue view/list`, issue/comment REST GETs; explicit host/repo. [G6][G10] |
| Publish/update spec and tickets, comments | `workitem create/update`, `workitem_comment create/update`; descriptions/comments support HTML. Repository conventions still govern formatting. [P4][P5][H] | Issue create/update and comment endpoints/CLI; bodies are not Plane HTML conventions. Some requested fields may be silently dropped without sufficient access: read back. [G6] |
| Parent and native blockers | `parent` UUID on create/update; `workitem_relation create` routes built-in `blocked_by`/`blocking` to the **dependencies** API; list returns grouped dependencies and custom relations separately. Not text links or the old generic relation resource. [P4][P5][P7] | Native sub-issues and issue-dependency REST endpoints exist. Installed `gh issue edit` supports `--parent`, `--add-sub-issue`, `--add-blocked-by`, `--add-blocking`; not an inference from a seed template. REST payloads use issue **IDs**, not display numbers. Host/version/access may differ. [G4][G5][G9] |
| Claim and progress/state | Assignees plus state update; `member me` resolves identity, state list resolves UUID/group. **No atomic exclusive claim/lease** is exposed by this inspected MCP surface. `manage_assignee` itself does read–modify–write. [P4][P5] | `--add-assignee @me` is assignment, not an exclusive claim. Issue open/closed state is not an arbitrary workflow/Projects status vocabulary. A repo must define its progress mapping. No exclusive claim guarantee established. [G4][G6] |
| PR and Retro publication | Tracker link/comment can point to a PR; Plane access is not Git-host PR capability. | `gh pr create/edit/view/list`, plus Git push; explicit repo/base/head and body file can publish spec/Retro links. Creation/edit are separate from merge; merge remains human-gated. [G3][G7][G12][map] |

Plane's server advertises consolidated **resource tools with an `action` argument**,
not one MCP tool per mutation. Tool-name filtering cannot make `workitem` read-only.
The PAT verifier checks `/users/me/` then supplies `read`/`write` scope labels; those
labels do **not** establish project write authorization. Actual API calls enforce
access. Plan/edition/role restrictions remain unverified. [P2][P3][P4][P5]

## Smallest credible native MCP client scope

**Standard facts** and minimum implementation consequences: [M1][M2][M3][M4][M5]

- Initialize with protocol version/client info/capabilities; negotiate a supported
  version and send `notifications/initialized`. Require the server's `tools`
  capability. Do not advertise sampling, elicitation, roots or experimental tasks
  unless implemented; explicitly reject unsupported requests without executing them.
- `tools/list` is paginated. Retain names/descriptions/`inputSchema` and available
  output schemas; refresh on advertised `notifications/tools/list_changed` (or
  reconnect/explicit refresh). Namespace by configured server to prevent collisions.
  Let models inspect discovered contracts, then invoke `tools/call(name, arguments)`.
  Direct model-tool advertisement or generic discover/call tools are design options;
  both need real discovery, validation and durable server/tool/schema identity.
- Preserve text/structured results, `isError` and JSON-RPC errors distinctly; bound
  content/catalogue sizes. Validate supported schema forms, reject unsupported
  content explicitly, and do not auto-fetch resource links or execute returned prose.
  Annotations are hints, not trusted authorization or idempotence guarantees.
- **Stdio-only first:** newline-delimited UTF-8 JSON-RPC on piped stdin/stdout,
  separately bounded stderr, initialized session lifetime, deadlines, shutdown and
  process-group cleanup. Launch approved executable/argv directly; apply explicit,
  narrow credential delivery after environment filtering, never via model arguments.
- **If native Streamable HTTP is chosen:** POST each message; accept JSON **and SSE**
  responses, track negotiated `MCP-Protocol-Version` and optional `MCP-Session-Id`,
  handle notifications/session expiry/shutdown. GET stream resumption is distinct
  from resubmitting a mutation. A plain POST-only JSON helper is insufficient.
- **If native OAuth is chosen:** protected-resource discovery (challenge/well-known),
  authorization-server metadata discovery, supported client registration, browser
  authorization-code flow with PKCE/resource binding, protected token persistence
  and refresh/error handling. Do not reuse model-provider tokens. Stdio gets local
  credentials instead; a preprovisioned PAT route can defer native OAuth, not claim it.
- Keep existing persist-before-effect/result-before-continuation behavior. Protocol
  support does not supply an approval policy; the tools spec recommends human
  confirmation/visibility. No vendor resource adapter, MCP registry browser,
  legacy SSE fallback or background task facility is necessary for this candidate.

## Nonmutating preflight versus write evidence

**Proposed preflight:** under the *actual product environment*, verify versions/targets,
MCP initialization/discovery, identity/workspace/project, full sample ticket/comments,
parent/blocker reads, states, git-host repository/base-branch and PR reads. Check
pagination/limits/errors with temporary offline fixtures. Never print tokens, headers,
raw config, HTTP traces or credential-store contents. For `gh api`, force GET for
reads: adding field parameters otherwise changes its default to POST. [H][G1][G2][G11]

A read proves that request's reachability/authentication/visibility then, not write
rights; public GET need not prove private access. Repository `permissions`/OAuth
scopes are useful but insufficient; GitHub has endpoint-specific fine-grained rights.
Later **authorized** writes need readback of intended fields/relations; no dummy
issue/PR or destructive cleanup as preflight. `gh pr create --dry-run` may still push. [G3][G5][G6][G7]

## Uncertain mutations, claims and retries

**Design constraints:** local termination/timeout/MCP cancellation does not undo a
remote commit. Bash can report Failed/Cancelled after an API write succeeded.
MCP cancellation may be ignored/arrive after completion; HTTP disconnect is not
cancellation. Durable recovery must distinguish “not sent” from “may have applied,”
even when the transport reports failure. [H][M1][M5]

Record intent/target and returned IDs/URLs before continuing. After uncertainty,
reconcile via reads: issue/comment markers or external IDs, exact dependency edges,
assignees/state, PR head/base and returned PR identity. Plane exposes external-ID
fields/filtering, but inspected sources establish **no uniqueness/idempotency-key
contract**; a marker is a search aid, not exactly-once protection. No generic MCP
request ID is a business deduplication key. Ambiguity requires human resolution,
not blind replay of create/comment/PR actions. Recheck assignments and blockers,
serialize local claim decisions; external writers can still race. [P4][P5][P6][P7][M1][M5][G7]

Bound safe-read/rate-limit retries; honor `Retry-After`, surface auth/permission errors.
Inspect lower-layer retries: SDK 0.3.1 defaults to `retry=None`; its optional policy
includes PATCH/PUT/DELETE, not POST. Do not infer safety from HTTP verbs or enable it
indiscriminately. GitHub recommends serial requests/spacing bulk mutations, not
idempotence. Read-before-write cannot eliminate concurrent duplication. [P6][G8]

## Minimum viable options to take back to HARNESS-6

1. **Process-only:** verified CLI/script (or generic MCP client executable), explicit
   auth/targets, operation inventory and honest recovery. Smallest product addition;
   this repo's MCP path is not proven, nor silently replaceable with raw API calls.
2. **Generic stdio tools client + bash:** candidate official Plane server plus `gh`;
   defer native HTTP/OAuth/other MCP features. New bidirectional process/protocol/
   config/durability and credential delivery, no vendor adapter; other servers possible.
3. **Generic Streamable HTTP tools client:** provisioned PAT/headers first if desired,
   OAuth separately. Less local packaging; more HTTP/session/auth/availability work.
   A pinned external bridge moves machinery outside the product, not trust/lifecycle costs.

Human choices: route/demo, fallbacks, auth provisioning, Setup repair/readiness and
minimal global approval boundaries. Unsupported capabilities stay visible; no implementation is selected.

## Primary sources

[question]: https://app.plane.so/reilley/browse/HARNESS-13/
[setup]: https://app.plane.so/reilley/browse/HARNESS-6/
[map]: https://app.plane.so/reilley/browse/HARNESS-1/
[prior]: https://github.com/Reilley64/harness/blob/7557855810075aab0436b7c24e2b4520fead3532/docs/research/main-flow-runtime-contracts.md
[H]: https://github.com/Reilley64/harness/tree/6b4512d6ef7b0aa38783ff939bfd0f76234f41a4

Paths under [H]: `docs/agents/{issue-tracker,domain}.md`, `docs/{tool-calls,security}.md`,
`infrastructure/src/{create_process/mod,tool_io}.rs`, `CODING_STANDARDS.md`.
MCP links name the versioned standard; source snapshot is identified above.

[M1]: https://modelcontextprotocol.io/specification/2025-11-25/basic/transports
[M2]: https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
[M3]: https://modelcontextprotocol.io/specification/2025-11-25/server/tools
[M4]: https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization
[M5]: https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation
[P1]: https://github.com/makeplane/plane-mcp-server/blob/5c3467adb36fe15a2ef8f9df6fb9f5dc24b6d881/README.md
[P2]: https://github.com/makeplane/plane-mcp-server/blob/5c3467adb36fe15a2ef8f9df6fb9f5dc24b6d881/pyproject.toml
[P3]: https://github.com/makeplane/plane-mcp-server/tree/5c3467adb36fe15a2ef8f9df6fb9f5dc24b6d881/plane_mcp/auth
[P4]: https://github.com/makeplane/plane-mcp-server/blob/5c3467adb36fe15a2ef8f9df6fb9f5dc24b6d881/plane_mcp/tools/workitem.py
[P5]: https://github.com/makeplane/plane-mcp-server/tree/5c3467adb36fe15a2ef8f9df6fb9f5dc24b6d881/plane_mcp/tools
[P6]: https://github.com/makeplane/plane-python-sdk/tree/721061d6f0d0047e431dd5fed66920d31296ba2b/plane
[P7]: https://github.com/makeplane/plane-python-sdk/tree/721061d6f0d0047e431dd5fed66920d31296ba2b/plane/api/work_items
[G1]: https://cli.github.com/manual/gh_auth_login
[G2]: https://cli.github.com/manual/gh_help_environment
[G3]: https://cli.github.com/manual/gh_pr_create
[G4]: https://github.com/cli/cli/blob/v2.102.0/pkg/cmd/issue/edit/edit.go
[G5]: https://docs.github.com/en/rest/issues/issue-dependencies?apiVersion=2022-11-28
[G6]: https://docs.github.com/en/rest/issues/issues?apiVersion=2022-11-28
[G7]: https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28
[G8]: https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
[G9]: https://docs.github.com/en/rest/issues/sub-issues?apiVersion=2022-11-28
[G10]: https://docs.github.com/en/rest/issues/comments?apiVersion=2022-11-28
[G11]: https://cli.github.com/manual/gh_api
[G12]: https://cli.github.com/manual/gh_pr_edit
