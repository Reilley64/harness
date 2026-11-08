# MCP Streamable HTTP and OAuth

## Supported protocol and configuration

HTTP uses the shared MCP client and ordinary typed configuration/tool loop, not a
second provider or tool coordinator. The supported HTTP protocol is 2025-06-18;
JSON and POST-response SSE are supported, not the deprecated endpoint-event SSE
transport. Optional unsolicited GET streams and resumable redelivery are not
implemented. Negotiated protocol/session headers accompany subsequent requests.
Sessions require bounded visible-ASCII identities; JSON-RPC identity and raw
result validation precede any public redaction. HTTP authorization failures and
uncertain sent calls are never automatically replayed. JSON requests/frames and SSE
events are bounded at 1 MiB; an SSE response has an 8 MiB, 4,096-line and 256-message
bound. Owned SSE decoding, JSON-RPC interpretation, metadata/schema conversion,
validation and public projection run on blocking workers. Once a correlated result
is admitted, its bounded validation/projection finishes even on Stop; projection
omission cannot erase a known Succeeded or Failed outcome.

Initialization and result validation use the 2025-06-18 contract. The bundled
`initialize_schema.json` retains the reachable initialization definitions from
[upstream revision 94f4ebc8ecb3764584bc30d5139d57c03692b734](https://github.com/modelcontextprotocol/specification/blob/94f4ebc8ecb3764584bc30d5139d57c03692b734/schema/2025-06-18/schema.json),
with descriptive prose omitted and validation keywords retained. Mandatory server
identity/capability fields are checked before discovery; server instructions are
not adopted as authority.

The 2025-06-18 result schema is distinct from OAuth discovery/client setup guidance
observed at the MCP 2026-07-28 authorization pages. Those web pages can move; their
observed content is not immutable historical approval or a latest-conformance claim.

Provisioned headers use the existing `HARNESS_MCP_…` references. HTTP framing,
session, cookie and origin headers are reserved. No header value is accepted from
the agent configuration tool. Endpoints require HTTPS except explicit loopback
HTTP fixtures, without userinfo, query or fragment. This resource restriction is unchanged.
Authorization-server endpoints may retain static routing queries, but cannot
preconfigure OAuth transaction/credential parameters that conflict with the
client's protected request. Issuer identifiers still forbid query/fragment.
Redirects do not forward tokens, and HTTP client retries are explicitly disabled
for MCP and authorization requests.

OAuth adds nonsecret configuration to an HTTP transport:

```json
{
  "name": "oauth-tools",
  "transport": {
    "type": "http",
    "url": "https://tools.example.invalid/team/mcp",
    "oauth": {
      "issuer": "https://accounts.example.invalid/tenant",
      "client": {
        "type": "registered",
        "client_id": "a-pre-registered-native-client",
        "redirect_uri": "http://127.0.0.1:38471/mcp/callback",
        "secret_reference": "HARNESS_MCP_CLIENT_SECRET"
      }
    }
  }
}
```

A public pre-registered client omits `secret_reference`. A confidential client
requires advertised `client_secret_post` support. Other client authentication
methods are not substituted. OAuth and a provisioned Authorization header cannot
be combined. The exact configured issuer must be advertised for the resource;
it is not inferred from the server name or a provider-specific login.

`client: {"type":"dynamic"}` requests public native registration only when a
registration endpoint is advertised. Dynamic registration is not universal.
`metadata_document` supplies an already hosted HTTPS `client_id` URL with a path
and an exact `redirect_uri`. Harness checks the document's client ID, name, redirect
binding and public-client authentication method; it never provisions hosting.
Client metadata URLs reject literal/encoded dot segments and documents containing
shared-secret fields. Their JSON media types may be `application/json` or a
registered `application/*+json` subtype.
Missing registration prerequisites block honestly. Registered/document clients
bind their configured IPv4 loopback port at `/mcp/callback`; dynamic registration
uses a new ephemeral port. A busy port is an actionable configuration failure.

## Explicit authorization and identity

Saving, registry discovery, reading and loading snapshots never log in. From the
exact-definition native Connect approval, the backend discovers metadata and
presents an operation-correlated protected browser link. A dedicated TUI pane keeps
it hidden: `o` opens it, `l` reveals/hides it, and Escape cancels the attempt.
Revealed links are paged with left/right arrows and reset on resize; protected
controls remain visible at small supported dimensions. Safe operation/configuration
diagnostics are likewise paged rather than clipped behind ordinary metadata.
Link notifications
bind the operation ID, workspace and definition revision. Replacement, dismissal
and cancellation invalidate the old presentation operation.

The resource is the complete endpoint including its path. Challenge metadata takes
priority; otherwise path-specific then root RFC 9728 well-known locations are
tried. Resource metadata must belong to the configured resource origin and its
`resource` must match the full configured URL. RFC 8414 and OIDC discovery use
path insertion and path appending in their specified order. Missing endpoints can
fall back; invalid/mismatched metadata is not silently bypassed. Authorization and
token endpoints come from exactly issuer-validated metadata, not origin guessing.

PKCE S256 must be advertised. Each attempt has unpredictable state and a fresh
verifier. Duplicate/malformed/stale callback fields and redirect/client/resource
mismatches are rejected before code exchange. A present `iss` must exactly match
the recorded issuer, even without advertisement and on error callbacks. When
advertised, `iss` is mandatory. This comparison never folds host/scheme, removes
default ports, normalizes percent encoding or changes trailing slashes. A denial
cannot act on unvalidated issuer information. Both authorization and token requests
include the resource indicator. Tokens are sent in headers, never in MCP URLs.

Private credentials live under `APP_DATA_PATH/mcp/auth`, independently of provider
authentication, editable definitions and conversations. Records bind the exact
workspace/definition revision, resource, issuer, client, redirect, token endpoint
and verified account for identity-bearing grants with a usable issuer userinfo
endpoint. Advertising userinfo alone does not turn a resource-only opaque token
into an identity credential. Opaque
tokens do not acquire invented subjects or unverified JWT identities. Identity-token claims are never used as unverified JWT identity. Identity-bearing
grants without supported account verification are rejected; raw identity tokens
remain private, bounded redaction inputs rather than identity evidence. A changed account requires
Disconnect and explicit new authorization. No model-provider credentials are reused.

Access tokens require a bounded positive expiry; indefinite lifetime is not
assumed. Refresh is on demand and never opens a browser. Short private-file locks
admit durable authorization/rotating-exchange leases under configuration admission;
bounded, cancellable lock waits do not hold network work. Admission generations and
active-grant generations reject stale refreshes/rejections even when an issuer
returns the same access-token string; network work happens outside those locks.
Stale completions cannot replace newer grants or Disconnect. An uncertain refresh
makes the old grant unusable across restart. Replacement credentials must be
saved before use, and admitted rotating exchanges/commits finish truthfully even
when their caller cancels. A storage error is an authentication failure. Individual private records are
bounded at 256 KiB; metadata, registration, userinfo and token response bodies are
bounded at 1 MiB and require their specified JSON media type. Signed metadata requires verification that this
profile does not implement, so it blocks rather than trusting unverified claims.
Canonical workspaces must be UTF-8; unsupported paths are rejected before trust
admission rather than colliding through lossy display names. Authorization links
are bounded at 64 KiB and require HTTPS or explicit loopback
without URL credentials or fragments. Callback queries are bounded at 16 KiB,
with a 600-second browser wait. Token
lifetimes are at most 365 days. The connection retains at most 256 distinct known
credential/session redactions before requiring reconnection. Native MCP requests
keep their backend response wait during cancellation and shutdown, so an admitted
credential/configuration commit is drained rather than abandoned by UI dismissal;
this does not introduce a Native Decider execution deadline.

## Lifetime and recovery

The backend runtime owns a workspace/revision slot shared by conversations. Closing
a panel does not revoke trust or disconnect ordinary trusted tools. Cancelling one
HTTP consumer drops its response wait, not a sibling's session. Explicit revocation
or definition changes remove registry eligibility and cancel obsolete authorization
attempts; queued requests recheck the exact slot before sending. Already admitted
consumers retain their accepted snapshot. Last-owner HTTP session DELETE is a
bounded best-effort cleanup, not proof that a remote server deleted its session.
There is no cross-process MCP daemon or exactly-once remote-effect guarantee.

A lost session during read-only discovery can initialize once and retry that read.
A sent `tools/call` is never retried. Cancellation, malformed responses and lost
transport after send produce Unknown and stop continuation under the ordinary
checkpoint/Resume policy. Authorization expiry/revocation/insufficient scope gives
safe Connect/configuration recovery, not a silent login or scope-upgrade loop.
A bounded insufficient-scope challenge is retained privately for the bound grant;
the next explicit native Connect requests its union with prior/resource scopes.
This never replays the failed tool call or opens a browser from ordinary tool use.

## Qualification boundary

Qualification follows the public configuration, ordinary model/tool/checkpoint,
HTTP/auth/callback/storage and stock SDK/native-launcher seams. Scripted loopback
resource, authorization-server, browser-user-agent and model-provider boundaries
are substitutions, not live authentication, account entitlement, human visual
signoff, independent acceptance or a whole-MVP gate. Focused compiler/TUI checks
alone do not establish those behaviors; exact-source behavioral evidence and CI
are separate. Historical stdio/Native/Setup closures remain external immutable
evidence, not fresh HTTP/OAuth results. No permanent tests or test-only product
routes are introduced.
