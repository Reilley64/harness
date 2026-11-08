# Worker requests, continuation and accounting

## Provider requests

OpenAI inference uses `POST https://api.openai.com/v1/responses` with the subscription
Bearer token, `store: false`, `stream: true`, and text history. Effort maps to
`low`, `medium`, `high`, or provider spelling `xhigh`.

OpenRouter uses its own key with `POST https://openrouter.ai/api/v1/responses`
through the same Responses HTTP/SSE decoder and existing application tool loop.
It sends the exact selected nested model ID, `store:false`, `stream:true`,
`provider:{allow_fallbacks:false,require_parameters:true}` and an empty plugin
list. No model fallback list, hosted advisor, paid connect probe or retry is added.
A returned OpenRouter model must match the requested ID or its exact catalog
canonical slug before accepting tools. Available terminal resolved-model and
serving-provider strings are bounded and retained privately on the model step;
absence remains unknown, not independently verified execution provenance.

Completed text pairs are replayed. Turns with real tool calls also retain their
original request, canonical provider continuation, steering, and honest function
results after interruption. Unfinished prose is excluded. No provider-side stored
response IDs, automatic mutation replay, or automatic retries are used. Across
provider or model changes, only ordinary assistant text and reconstructed function
requests/results are replayed: private reasoning, output item IDs and original
provider call IDs are omitted. Function request/result pairs use recorded local
execution IDs for this translation. Compatible same-provider/model continuation
retains its original opaque items only while its generation-time catalog canonical
identity still matches, even if the response reported the unversioned requested ID.
Missing historical canonical ownership cannot establish compatibility with an
available canonical identity. A canonical change between body preparation and HTTP
admission rejects the request before sending; it does not transmit the old private
context or automatically retry. Safe snapshots expose message/run provider IDs,
not private continuation.

SSE parsing handles fragmented UTF-8, CRLF, multiline data, and keepalive comments.
Content types are compared case-insensitively. If the header is missing, empty, or
unusable, the body must still decode as Responses SSE and reach a terminal event;
headerless JSON-like, HTML/XML-like, empty, and non-SSE bodies fail with sanitized
diagnostics. Explicit non-SSE media types remain rejected. No raw headers or
response bodies are included in these diagnostics.
Text/refusal deltas and bounded tool summaries are presented. A missing terminal response or
provider error fails the reply while preserving partial text. Refusal and
incomplete responses remain distinct terminal statuses. Raw provider error bodies,
credential tokens, and reasoning do not cross ACP.

Provider input/output token reports are saved independently of continuation text
and survive resume. Each new assistant reply records a bounded generation ledger:
an unknown entry is checkpointed before inference, then filled only with reported
counts. Session totals include reported superseded/cancelled generations and tool
continuations, even when their content is excluded from future context. Missing
reports remain unknown or a lower bound; steering alone no longer implies a gap.
A pending generation reservation is removed only when the adapter positively
reports that no inference request was attempted. This preserves previous reported
usage and latest context; attempted failures and crash uncertainty stay unknown.
Legacy replies without a ledger use retained per-step reports conservatively.
Safe cumulative and latest-request context counters cross ACP after checkpointing.
Optional model context-window metadata is retained in conversation and preference
snapshots; it is not a ChatGPT account quota.

Settings are embedded, with environment-only overrides:

- `REPLIES__OPENAI__BASE_URL`: `https://api.openai.com/v1`; HTTPS is required except
  loopback HTTP fixtures.
- `REPLIES__OPENAI__TIMEOUT_SECONDS`: 600; must be nonzero. Applies to inference,
  not to abandoning a credential-refresh commit.
- `REPLIES__OPENROUTER__BASE_URL`: `https://openrouter.ai/api/v1`; its origin must
  match the configured/stored OpenRouter issuer before credentials are used.
- `REPLIES__OPENROUTER__TIMEOUT_SECONDS`: 600; nonzero, cancellable inference.
- `APP_DATA_PATH`: `~/.harness`, the only storage-root setting.

Credential resolution and on-demand refresh are described in
[provider connections](../provider-connections.md). A cancelled refresh wait stops
promptly; once a rotating refresh exchange begins, cancellation waits for its
verified replacement to be saved, then skips inference. There is no refresh daemon.
