# Shared MCP configuration and stdio tools

Harness owns MCP configuration independently of model-provider connections. The
native `/mcp` command and the `mcp_configuration` agent tool use the same public
`update_mcp_configuration` application use case/port, effective configuration and
connection runtime. Configuration does not require a conversation, catalog,
provider credentials or inference. All consumers of one backend runtime share
these tools; there is no role-based permission matrix.

This implementation executes **stdio only**. Custom HTTP definitions can be
saved, validated and reviewed, but their status is `http_execution_unavailable`.
Connect does not fabricate an HTTP connection or browser authentication. The
Streamable HTTP/OAuth adapter is separate work.

## Native workflow

- `/mcp`: inspect effective server names, source, status and revision.
- `/mcp reload`: validate manual edits and replace the effective snapshot.
- `/mcp save <nonsecret JSON definition>`: add/replace one app definition using
  the last observed configuration revision. Open `/mcp` first if no revision has
  been read. Close the panel with Escape before entering another command.
- `/mcp remove <name>`: remove an app definition, not a repository override.
- `/mcp registry <search>`: optionally read registry metadata. `v` opens paged
  metadata; it is untrusted data, not installation/launch authority or a complete
  login recipe. Package names, required/secret flags and documentation links are
  hints for a definition the user/agent prepares, never automatically executed.

In the panel, Up/Down selects a server, `c` reviews Connect, `d` revokes local
trust and retires the connection, `t` discovers tools, and `r` reloads. Exact
nonsecret definitions are reviewed in ASCII-escaped pages with Left/Right;
approval is enabled on the last page. Resize returns review to the first page.
Enter explicitly approves that exact revision. Escape cancels review without
activation, or closes the panel/cancels the wait. An already completed save or
trust commit is not rolled back by dismissing a pending operation. Pre-cancelled
operations and cancelled configuration-mutex waiters start no persistence or
activation. Save/remove and protected trust updates check cancellation again at
write admission; after admission, the commit and snapshot completion are awaited
rather than abandoned or rolled back. Connect also checks cancellation before
process launch, so a saved trust decision need not imply successful discovery.
Read/reload to reconcile a completed commit whose discovery or response was lost.
Conversation state and the editable command draft are not submitted as chat by these controls.

Connect records local trust and initializes/discovers a stdio server. Subsequent
`trusted` status describes consent/eligibility, **not a health or liveness
claim**; discovery/execution reports current failures. Closing the panel does not
disconnect tools used by other conversations. A failed executable/runtime or
missing provisioned input requires correcting configuration or following the
server's documentation, not silently installing a package or restarting Setup.

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
This schema does not yet configure browser OAuth. Saving an HTTP reference does not resolve credentials or send a request to its
endpoint. Connect fails as unavailable and records no HTTP activation consent;
the dependent adapter must obtain its own explicit native Connect/login.

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
a private persisted snapshot across restart. With no previous valid snapshot the
effective set is empty and visibly stale. Fix the original file and reload.
Read/status does not launch a server. New backends do not start tools merely by
loading a persisted snapshot; an explicit trusted tool/discovery request is needed.

## Protected trust and provisioned secrets

Trust is user-local, never accepted from a repository file, registry metadata or
agent configuration-tool arguments. It is stored privately under
`<app_data_path>/mcp/trust.json`, separate from provider auth. Consent binds the
canonical workspace, source and exact definition revision (including command,
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

Connections belong to the backend runtime, not a single conversation. Concurrent
consumers of the same workspace/revision share one process; requests serialize on
that connection. A conversation finishing does not tear down another consumer's
server. Reload/removal/revocation retires obsolete registry ownership; already
executing consumers retain their accepted snapshot to completion, while queued
calls recheck eligibility before sending. The last held connection reference
closes/kills its process group; dropping the last backend/runtime consumer cleans
up remaining children. This is in-process sharing, not a cross-process MCP daemon.
Unix process groups do not contain descendants that deliberately escape them.

Initialization is bounded at 30 seconds; operations at 120 seconds. NDJSON frames
are bounded at 1 MiB, tool discovery at 16 pages/256 unique tools/1 MiB metadata,
and individual schemas at 64 KiB. Owned frame bytes move to a blocking decoder;
public metadata/results move to blocking workers with an immutable shared
redaction context, without an async-worker copy of the frame or secret set.
Cancellation can stop waiting while those workers safely own their bounded
inputs. JSON-RPC correlation, initialization and original argument schemas are
interpreted before redaction. Tool results are moved rather than deep-copied on
the async worker. A tool name containing a provisioned value is unavailable,
not silently renamed into a different executable identity.

Public projection scans original UTF-8 text and keys from left to right, choosing
the longest matching provisioned value at each position (lexical order breaks
length ties). Equal provisioned values are deduplicated; short values and repeated
environment references remain valid. The static `[redacted]` marker is never
scanned again. Its characters may themselves equal short provisioned values.
Projection charges encoded bytes before appending text, keys or container content:
1 MiB per result, 64 KiB per schema, and 1 MiB aggregate encoded discovery metadata,
including escaped schema strings and field overhead. Each projected JSON value
also has a 16,384-node and 64 MiB byte-comparison budget. Serialization checks both
its own bytes and outer JSON-string escaping before extending its output buffer.
These construction limits bound transient string/container growth, not just the
finished output; an abandoned worker cannot continue exponential replacement.
Discovery rejects over-budget metadata or colliding projected keys before any
tool call is sent. Descriptions still retain at most 4,096 projected characters.

Tool-result validation and encoding also run on a blocking worker. Supported
text, image, audio, resource-link and embedded-resource blocks follow the bundled
2025-06-18 MCP contract. Embedded resources require URI plus text or base64 blob;
links require name and URI. Optional known metadata/annotations and field types
are checked, including URI/base64 formats, without rejecting extension fields.
The bundled `infrastructure/src/mcp/content_schema.json` is the reachable
`CallToolResult` definition subset of the upstream
[2025-06-18 schema](https://github.com/modelcontextprotocol/specification/blob/main/schema/2025-06-18/schema.json),
with descriptive prose omitted but validation keywords preserved. Schema formats
are asserted, including a local base64 `byte` validator; no schema retrieval is
enabled. Malformed content is not evidence of success or a confirmed failure.

The original tool result is validated and its typed outcome fixed before public
redaction. Known provisioned values overlapping `isError`, optional-field names,
URI strings or base64 data cannot change Failed/Succeeded/Unknown. Redaction
still protects known values and object keys in durable/model-visible output.
That output is a sanitized observation, not an unchanged or replayable MCP
protocol envelope; its redacted fields are not reinterpreted to select status.
If safe projection or encoding reaches a budget, or distinct object keys project
to the same key, the result content is omitted with an honest bounded explanation
and `truncated: true`. The already validated Failed or Succeeded status is
preserved, not converted to Unknown or its opposite. This omission neither retries
the tool nor authorizes replay. Malformed raw content still becomes Unknown.

Unsupported client requests are rejected; no
sampling, elicitation, roots, shell, package install or browser login capability
is granted by a server request. Explicit remote tool errors/rejections are Failed.
Cancellation, disconnection, timeout or malformed protocol **after sending a
call** records Unknown and stops the turn: effects may have occurred. Reconcile
using the repository's documented tracker/CLI procedures before repeating. Before
send, failures cannot claim execution. Reads/status/restart never replay calls.

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
tracker writes/claims, HTTP/OAuth and later factory roles remain separate evidence.

The cycle-2 rework retains original historical probe claims separately from fresh
focused checks. Public configuration/typed-call probes cover pre-cancelled and
queued mutations, an admitted completed save, compiler-cause retention with safe
diagnostics, current-thread runtime responsiveness, frame/correlation bounds,
and exact malformed-resource cases with valid controls. Focused stock ACP
composition also uses the accepted Griller/CoS, both provider factories, positive
Chat/Decisions catalog capabilities, allowlist, schema-6 checkpoints and obsolete
assignment authority gates. Whole-project build/test/check is CI-owned under the
[root policy](../AGENTS.md#verification), not a local acceptance claim.
