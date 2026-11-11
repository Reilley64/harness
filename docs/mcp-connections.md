# Shared MCP configuration and tools

Harness owns MCP configuration independently of model-provider connections. The
native `/mcp` command and the `mcp_configuration` agent tool use the same public
`update_mcp_configuration` application use case/port, effective configuration and
connection runtime. Configuration does not require a conversation, catalog,
provider credentials or inference. All consumers of one backend runtime share
these tools; there is no role-based permission matrix.

Stdio and Streamable HTTP use one protocol client, typed invocation path and
bounded result projection. HTTP selects MCP protocol **2025-06-18**, not legacy
HTTP+SSE or universal/latest conformance. The result contract remains the bundled
2025-06-18 schema. See [HTTP and OAuth](mcp-http-auth.md) for supported prerequisites,
authorization identity, refresh, recovery and current qualification limits.

## Native workflow

See [native MCP controls](frontend/configuration.md#mcp-controls) for review,
Connect/cancel, reload, protected input and exact-key behavior.

## Editable definitions and merge rules

App defaults: `<app_data_path>/mcp/servers.json` (`APP_DATA_PATH` defaults to
`~/.harness`). Repository overrides: `<saved workspace>/.harness/mcp.json`.
Both use this schema; app files/directories must be owner-only (0600/0700).
Repository definitions may be committed, but never credentials or trust decisions.

```json
{
  "version": 1,
  "servers": [
    {
      "name": "internal-tools",
      "enabled": true,
      "transport": {
        "type": "stdio",
        "command": "/absolute/path/to/an-installed-server",
        "args": ["--stdio"]
      },
      "environment": {
        "SERVICE_TOKEN": "HARNESS_MCP_INTERNAL_TOKEN"
      }
    }
  ]
}
```

`command` is an executable, and `args` is an argument array. Harness does not
shell-evaluate registry input, install packages or invoke a package manager by
itself. A user may explicitly trust a definition that invokes an installed
package runner, but registry discovery alone never grants that permission.
Commands and arguments must be nonsecret; provision values out of band instead.

A custom HTTP definition is independent of registry publication:

```json
{
  "name": "private-http",
  "transport": {
    "type": "http",
    "url": "https://tools.example.invalid/mcp",
    "headers": { "Authorization": "HARNESS_MCP_PRIVATE_AUTHORIZATION" }
  }
}
```

HTTP URLs require HTTPS (loopback HTTP is allowed), and cannot embed credentials,
a query or fragment. Headers contain provisioned references, not literal values.
Saving an HTTP reference does not resolve credentials, contact its endpoint or
grant trust. Configured generic OAuth additionally requires explicit native Connect;
see [HTTP and OAuth](mcp-http-auth.md) for the nonsecret `oauth` configuration.

Names are stable, case-sensitive, 1–64 ASCII alphanumeric/underscore/dash/dot
characters. Repository entries replace an entire app entry of the same name,
not individual fields. Additional names add servers. An explicit repository
`{"name":"internal-tools","enabled":false}` disables an app default. Duplicate
names within either file, unsupported versions, invalid/unknown definition fields,
invalid transports and excessive files/entries are errors, not last-entry-wins.
Environment/header values must be `HARNESS_MCP_…` variable references. Unknown
**top-level** metadata is retained by app edits; edited server definitions follow
the strict schema. Unrelated server entries and their ordering are preserved.

Files are bounded at 256 KiB and 128 servers each. Save/remove uses the revision
of both observed files, an owner-only cross-process app lock, fresh validation,
a second content comparison, private same-directory temporary writes, sync and
atomic replacement. Stale/conflicting edits fail without overwriting them; reload
and reconcile rather than resending blindly. Editors should honor the app lock
(`servers.lock`) or finish edits before saving. As with ordinary file tools,
advisory locks cannot eliminate a noncooperating editor's final rename race.

Startup (native workspace initialization or first backend configuration access)
and explicit reload validate both files without rewriting them. Invalid input
shows its app/repository source and retains the last validated snapshot, including
a private persisted snapshot across restart. Configuration outputs privately own
the original validation/storage diagnostics, including failed saved-snapshot
recovery; ACP and model-facing summaries publish only safe contexts, not those
causes. A later successful reload clears the stale diagnostics. With no previous valid snapshot the
effective set is empty and visibly stale. Fix the original file and reload.
Read/status does not launch a server. New backends do not start tools merely by
loading a persisted snapshot; an explicit trusted tool/discovery request is needed.

## Protected trust and provisioned secrets

Trust is user-local, never accepted from a repository file, registry metadata or
agent configuration-tool arguments. It is stored privately under
`<app_data_path>/mcp/trust.json`, separate from provider auth. Consent binds the
canonical UTF-8 workspace, source and exact definition revision (including command,
arguments, endpoint and environment/header references). A new/materially changed
definition requires another native Connect. Approval replaces the prior decision
for that stable workspace/name; reverting after approving a different definition
also requires review. Disconnect removes that decision. Trust storage is bounded
at 1,024 workspace/name decisions; it is not an executable permission sandbox.

Provision secret values through the launching process environment, using only
explicit `HARNESS_MCP_…` references. No secret-entry-in-chat or agent-tool argument
path exists. Configure references, not values; do not put credentials in commands,
URLs or argument arrays. There is no machine-wide credential installation and no
model-provider token reuse. A stdio child receives only PATH plus its explicitly
referenced environment inputs, not HOME, provider/cloud credentials or the parent
application configuration. Required missing references fail before launch.

Raw child stderr is discarded. Known provisioned values are redacted from parsed
stdio metadata/results (including JSON object keys) before they can enter native
presentation, durable history or model continuation. Errors use safe contexts,
not arbitrary remote error messages. Original parser and schema compiler errors
remain internal causes beneath safe context; application tool-result and ACP
projection use the safe diagnostic boundary from [security](security.md), never
format the cause chain. This is not universal secret detection:
a trusted process can read OS-user files, transform values or communicate by
other means. Existing bash/file tools also remain unsandboxed. The local trust
file and ACP client are trusted same-user surfaces, not protection against a
malicious same-user process writing files or impersonating native ACP requests.
See [security](security.md).

## Typed invocation, lifetime and uncertainty

`domain::Tool` adds fixed `McpConfiguration` and `McpCall` identities rather than
a generic plugin dispatcher. External JSON decodes into `McpConfigurationAction`
or `McpCallArguments` before application execution. Dynamic server/tool identities
are explicit fields of the MCP invocation, not arbitrary built-in tool names.
`mcp_configuration` read returns source/status/revisions and definitions;
`discover` returns advertised names, descriptions and input schemas.

Invoke `mcp_call` with `server`, `tool`, exact server `revision`, and an
`arguments` object matching that advertised schema. The adapter verifies the
current effective revision/trust, initializes MCP, refreshes `tools/list`, and
validates arguments before `tools/call`. JSON Schema network/file retrieval is
not enabled. The provider sees MCP discovery as ordinary untrusted tool data.
Agent configuration supports read/reload/save/remove/discover/registry, **not**
Connect or Disconnect. Saving is not activating.

Both invocations follow the existing application loop: requested calls and
Running are durably checkpointed before configuration/execution effects, terminal
results are persisted before provider continuation, and ordinary output/artifact
bounds apply. `TOOLS__ENABLED=false` suppresses/rejects both agent tools without
disabling the user's native configuration UI. MCP calls conservatively count as
mutating for crash recovery. There is no automatic mutation retry.

Connection lifetime, framing, raw-before-projection validation, bounded redaction
and uncertain sent effects are specified in the [result contract](mcp/result-contract.md).

## Registry and configured CLI fallback

Optional registry search performs a credential-free, bounded GET against
`MCP__REGISTRY_URL` (default
`https://registry.modelcontextprotocol.io/v0.1/servers`) with `search`/`limit`
parameters. HTTPS or loopback HTTP is required and redirects are not followed.
Metadata is displayed/returned as data; selection means preparing an explicit
custom definition with the same save schema, not granting launch permission.
Missing/incomplete/unreachable metadata leaves the manual file/custom-command
and server-documentation paths available. No registry membership is required.

Prefer the repository's documented CLI/bash tracker/git-host path where that is
the configured procedure. MCP adds generic transport, not a vendor-specific
tracker adapter or automatic replacement workflow. Successful discovery/read
proves neither write access nor an atomic exclusive issue claim. Unsupported or
rejected writes remain failed/blocked; ambiguous effects require reconciliation.
Actual authentication, external provisioning and real tracker mutations need
their separately established authority.

## Verification boundary

Retained temporary external probes exercise stock ACP/native launcher paths,
real config/registry/stdio adapters and the normal typed invocation loop. Controlled
loopback model Responses and explicitly fake provider records substitute provider
inference/authentication only. Public application read-port observations check
Running before effects and stored results before continuation. Native PTY checks
cover review/cancel, custom save, unavailable registry, reload, resize and terminal
cleanup; they are not a human terminal-emulator visual signoff. No permanent tests,
test-only dependencies or production test hooks are added. Live servers, actual
tracker writes/claims and later factory roles remain separate evidence. HTTP and OAuth
observations are retained separately: historical stdio observations below are not
HTTP/OAuth behavioral acceptance.

The cycle-2 rework retains original historical probe claims separately from fresh
focused checks. Public configuration/typed-call probes cover pre-cancelled and
queued mutations, an admitted completed save, compiler-cause retention with safe
diagnostics, current-thread runtime responsiveness, frame/correlation bounds,
and exact malformed-resource cases with valid controls. Focused stock ACP
composition also uses the accepted Griller/CoS, both provider factories, positive
Chat/Decisions catalog capabilities, allowlist, schema-6 checkpoints and obsolete
assignment authority gates. Whole-project build/test/check is CI-owned under the
[root policy](../AGENTS.md#verification), not a local acceptance claim.
