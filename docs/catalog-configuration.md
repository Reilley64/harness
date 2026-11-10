# Catalog configuration

Catalog entries describe identities and metadata, not credentials, account
entitlement or executed work. Positive Chat capability is required independently
for workers; native Decisions selection has its [own contract](native-decider.md).

## Provider configuration

Defaults are embedded from `application.toml`. Runtime overrides use environment
variables only, with **no prefix** and `__` separating groups:

```sh
PROVIDERS__URL=https://your-endpoint.example/providers.json \
PROVIDERS__MEMORY_TTL_SECONDS=300 \
APP_DATA_PATH=~/.harness \
./bin/harness
```

The default URL is `http://localhost:3000/providers.json`. The endpoint should
return a JSON array:

```json
[{ "id": "example", "name": "Example Provider" }]
```

## Model configuration

Models use the same reader and cache composition, with defaults embedded from the
`[models]` section of `application.toml`. Overrides are `MODELS__URL`,
`MODELS__MEMORY_TTL_SECONDS`, and the shared `APP_DATA_PATH`. The URL defaults to
`http://localhost:3000/models.json`, TTL to 300 seconds, and cache to
`<app_data_path>/models.json`.

The endpoint returns an array of objects with `id`, `name`, and `provider_id`:

```json
[
  {
    "id": "example-model",
    "name": "Example Model",
    "provider_id": "openai",
    "efforts": ["low", "medium", "high", "extra_high"],
    "pricing": {
      "input": 1.25,
      "cached_input": 0.25,
      "output": 10
    }
  }
]
```

The illustrative metadata example omits capabilities; it does not authorize a
worker. Positive `chat` capability and current backend eligibility are separately
required.

The `read_models` use case reads through the `ReadModels` port. JSON serialization
stays in the infrastructure DTO; the dependency builder converts DTOs into domain
models. `efforts` is optional and defaults to no effort control; the picker only
offers the listed levels, preferring medium when supported. `pricing` and its
individual fields may be omitted or null. Prices above are illustrative USD per
million tokens, not published rates for an actual model. A numeric zero means
free; missing means unavailable.

`server/models.json` is a static Harness-owned catalog, not a live provider
inventory or runtime OpenRouter metadata import. Its four original direct-OpenAI
rows retain OpenAI's published Standard, short-context USD rates per million
tokens, verified on 2026-11-06. The 168 curated OpenRouter rows come from public
metadata captured on 2026-11-07 for OpenAI, Anthropic, xAI and TypeSafe, including
the verified TypeSafe latest alias; their listed per-token prices were converted
to USD per million tokens. Those rows are not covered by the direct-OpenAI
verification date or Standard short-context pricing claim. These snapshots do not
prove entitlement, current availability or actual billing. Long-context requests,
processing modes, cache writes and other fees may differ. See
[pricing sources and limitations](research/model-pricing.md).

## Catalog caches

The closure-backed catalog ports use the infrastructure HTTP reader composed with
`with_memory_cache` and `with_file_fallback`. Each adapter owns its response/cache
representation. `MODELS__URL` is the sole model source: there is no secondary
OpenRouter request, import, merge or fallback. Obsolete extra environment settings
have no effect; existing secondary cache files are neither read nor removed.

- Port clones share a memory cache, with a 300-second default TTL. A miss or expiry
  fetches the configured URL with a 10-second timeout; concurrent reads share one
  refresh.
- Successful responses update memory and atomically replace the optional private
  disk cache. Persistence failures retain fetched data and emit only a fixed
  tracing warning, with no raw error, URL, path or response content.
- Connection, HTTP and JSON failures fall back to disk regardless of cache age.
  Fallback results enter memory for the configured TTL. If both sources fail,
  return an error; an empty array remains a valid result.
- Remote bodies, fallback files and encoded caches each have a 16 MiB limit,
  checked before parsing or growing the encoded buffer past the limit. Advertised
  lengths and streamed chunks are bounded. Parsing, encoding, memory-cache cloning
  and domain conversion run on blocking workers. Unsafe or oversized fallback
  files are rejected.
- Default disk paths are `~/.harness/providers.json` and `~/.harness/models.json`,
  derived from `app_data_path`. A leading `~` expands from `HOME`; relative
  configured paths resolve against the working directory.

Public catalog metadata does not prove entitlement, quality or execution.
Catalog reads and provider connects never invoke a model or refresh the static
snapshot from OpenRouter.

## Routing evidence observations

The sole `MODELS__URL` source may optionally attach `routing_evidence` to a
curated row. Existing rows need not change and missing observations stay unknown.
This is advisory source evidence, never a second inventory or permission source.
For each observation provide:

- `category`: `catalog`, `capability`, `pricing` or `benchmark`;
  `metric`, nullable finite `value`, and `unit`.
- `source`, `source_url`, `revision`, and nullable Unix-second
  `measured_at`, `published_at`, `effective_at`. Retrieval time is assigned by
  Harness on the successful fetch, not substituted for any original source date.
- Exact `requested_model`, nullable `resolved_model` and `serving_provider`,
  nullable `effort` (`low`, `medium`, `high`, `extra_high`) and
  `configuration`. Benchmark measurements require an original measurement date,
  nonempty units and configuration; they must not be inferred from other efforts.

Pricing values must be nonnegative and use `USD per million tokens`; no
implicit conversion or subscription incremental-cost inference occurs. Preserve
primary-source attribution and applicable licenses when publishing observations.
There are at most 32 observations per row; source bodies and private caches are
bounded at 16 MiB. Invalid sourced observations are a source failure, not silently
accepted current evidence. The source adapter uses HTTPS or explicit loopback
fixtures, no URL credentials, redirects or retries, and no provider credentials.

See [native selection](native-decider.md#sourced-evidence-and-refresh) for the
24-hour/seven-day freshness rules, on-demand refresh and immutable decision
inspection. Refetching the static source does not turn undated list rates or
missing benchmarks into fresh measurements.

## Reading and caching

See [Catalog caches](#catalog-caches) for refresh, fallback and bounds.
Local verification follows the [focused-check/CI policy](ci.md).

## Provider authentication settings

Defaults are embedded in `application.toml`. Environment overrides include:

- `APP_DATA_PATH`: shared app data directory, default `~/.harness`.
- `AUTHENTICATION__OPENAI__OPEN_BROWSER`: whether Rust opens the browser automatically.
- `AUTHENTICATION__OPENAI__TIMEOUT_SECONDS`: login timeout, default 600 seconds.
- `AUTHENTICATION__OPENAI__DYNAMIC_CLIENT_ID`: initial-registration client ID,
  default `dynamic_agent_client`.
- `AUTHENTICATION__OPENAI__RESOURCE`: OAuth resource, default
  `https://api.openai.com/v1`.
- `AUTHENTICATION__OPENAI__SCOPES`: space-separated requested OAuth scopes.
- `AUTHENTICATION__OPENAI__ISSUER`: normally `https://auth.openai.com`; override only
  for a trusted environment. HTTP is accepted only on loopback for local fixtures.

The provider [connection contract](provider-connections.md) owns authentication,
credential storage, cancellation and OpenRouter origin binding. These environment
names are configuration, not permission to log in or run paid inference.
