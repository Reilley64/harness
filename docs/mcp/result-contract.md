# MCP connection lifetime and result contract

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
