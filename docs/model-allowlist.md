# Model allowlist

## Scope and settings

Harness has one durable, user-local positive model allowlist, shared across
workspaces and conversations. Open `/model` and press Ctrl+A to edit it in the
existing model picker. Enter saves; Ctrl+U clears the draft; Escape discards the
draft without changing policy. Patterns are separated by commas or whitespace.
An empty saved list permits no inference. Closing settings never retries a
blocked request, picks a replacement, signs in, or broadens policy.

Patterns have the form `provider/model`. Split on the **first** slash: in
`openai/team/model`, the provider is `openai` and the model ID is `team/model`.
Only `*` is special; it matches zero or more characters in its component,
including slashes in the model ID. Matching is case-sensitive and covers the
entire provider and model components. Examples:

- `openai/*` permits all model identities for that provider, not other providers.
- `*/team/*` matches a nested model prefix across providers.
- `openai/team/model` matches that exact nested identity.
- `openai/gpt-*` also covers future matching identities; those still require an
  available worker protocol, catalog selection/effort and valid authentication.

Patterns combine with **OR**, not precedence. Removing a specific positive
pattern does not revoke an identity still covered by `openai/*` or another broad
positive. Replace/remove broader coverage to narrow. There are no deny rules;
`!` is rejected. Other glob syntax has no special meaning. No repository,
agent-role or conversation setting can widen this application-wide policy.

The picker shows blocked models and a safe reason. Unsupported provider worker
protocols are capability blocks, missing positive coverage is an allowlist
block, and missing/invalid saved connections are authentication blocks. Supported
effort and current catalog selection are validated on selection/session creation,
on sends with either explicit or restored selection, and at every worker-step
boundary. A stale saved model/effort cannot bypass capability validation.
A catalog entry is not account entitlement, a supported protocol, or permission.
Actual provider refusal, access and usage failures remain visible; Harness does
not infer remote entitlement from the catalog or silently switch models.

## Initial setup, reconnect and legacy connections

Before any policy exists, inference is blocked. Successful provider setup adds
that provider's wildcard once to the initial, uncustomized policy. Opening a
catalog, loading history or discovering legacy credentials does not add coverage.
A legacy connected user with no policy can explicitly sign in or edit settings.

Saving any custom list, including an empty one, marks the policy customized.
Subsequent authentication never broadens that policy, including first
authentication after a user has already saved a custom list. This conservative
rule also applies to later provider setup: a customized policy needs an explicit
settings edit to permit that provider. Re-authentication of an uncustomized
provider does not restore removed coverage. Initialization and settings updates
share the same locked atomic write path, so a concurrent customization is not
lost to successful sign-in.

Authentication failure/cancellation does not initialize coverage. Credentials
are saved before policy initialization; a failure/cancellation at either durable
commit point can already have saved credentials or policy. Reload their public
state rather than assuming a rollback. Saving settings alone never logs in or
runs inference.

## Inference boundaries and existing effects

The application validates current policy and saved connection when selecting a
model or creating a legacy conversation or new durable Griller. Existing Grillers
remain inspectable while blocked; sends and explicit Resume share the admission
coordinator. Validation also precedes every model step: initial generation, tool
continuation, steering restart, and final budget summary. Policy is reloaded from disk, not a process-local catalog cache.
The provider-bound OpenAI adapter checks it again immediately before the
Responses HTTP request, after any credential refresh and request preparation.
A policy write after that admission cannot recall an already-started request.

Revocation does not cancel an in-flight inference or an already-started tool.
Their text, outcomes and reported usage remain authoritative. After the current
effects/results are checkpointed, disallowed continuation stops with an explicit
configuration error and no replacement model or retry. The assistant is saved
with its partial content and failed status; existing successful/uncertain tool
outcomes are not rewritten or replayed. A pre-attempt block adds no fictitious
generation usage. The adapter positively distinguishes failure before a request
attempt from an attempted request with no report. Only a known-unattempted pending
generation reservation is removed; previous reported totals/latest context remain
intact. An attempted failure or process-killed reservation remains unknown, never
a fabricated zero. A request admitted before a concurrent change may still finish;
unknown reports remain unknown.

The existing closure-backed ports and provider-bound factory remain in place.
The Serde-free domain `ModelPattern` and `ModelAllowlist` contain the positive
matching rule plus customization and initialize-once transitions. Storage owns
record conversion, locking and atomic replacement, not permission decisions.
Application admission is shared by workers and model settings.
Future native Decider/OpenRouter slices can reuse those values and policy ports;
neither provider nor native decision inference is implemented here.

## Storage and public interfaces

Policy is stored at
`<app_data_path>/preferences/model-allowlist.json`; the only root override is
`APP_DATA_PATH`. The version-one record owns patterns, a customization marker,
and initialized provider identities. Missing means no permission, not a default
model. Unsupported/corrupt/unsafe records fail closed and are not overwritten,
even by reconnect or a settings update.

There are at most 256 patterns, each at most 512 UTF-8 bytes, with nonempty
provider/model components and no whitespace/control characters. Records are
bounded at 256 KiB; parsing, validation and encoding run on blocking workers.
Updates use an owner-only no-follow lock, bounded private file reads, private
same-directory temporary writes, atomic rename and directory sync. Concurrent
writers fail explicitly instead of silently overwriting another in-flight
update. Records remain owned independently by each infrastructure port adapter.
As with existing private storage, Unix is required.

ACP advertises `harness/modelAllowlist: true` and exposes:

- `_harness/model-allowlist/read`, parameters `{}`.
- `_harness/model-allowlist/update`, parameters `{"patterns":["openai/*"]}`.
  This replaces the complete positive list, not an incremental allow/deny edit.
- Both return `{"patterns":[...]}`; validation/storage failures use `-32006`
  and a safe diagnostic. Unknown fields are invalid parameters.
- `_harness/models` adds nullable `blocked` diagnostics from current local policy,
  connection and worker protocol checks. A null value is local admission
  readiness, not a guarantee that the provider/account will serve the request.
  Historical/preference model snapshots do not establish current eligibility.

Settings reads need no credentials, catalog, session or inference. Loading reads
are cancellable on Escape, picker reload/close, disconnect and unmount; late
results cannot reopen a dismissed editor. Committing writes remain distinct and
are drained at shutdown rather than pretending dismissal rolls back a commit. These are
ordinary application use cases and ACP operations, not verification/debug APIs.

## Verification boundary

Temporary external probes exercise stock ACP/application/composition, atomic
persistence, signed loopback OAuth verification and refresh, real OpenAI
Responses HTTP parsing/tool continuation, native file/process effects, and
actual launcher/Ink controls in a PTY. Provider responses and the identity server
are scripted local fixtures, not a real account or paid inference. The probes
cover overlapping positives, nested IDs, restart and legacy policy behavior,
empty/custom policy through re-authentication, inaccurate entitlement avoidance,
configuration/auth cancellation, policy changes during tools/in-flight inference
and credential refresh, and retained outcomes/usage. Native evidence is terminal
bytes and public state observations, not human visual acceptance. Sources and
sanitized command evidence are retained outside the checkout by the implementer.
