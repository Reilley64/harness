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

Model/provider catalogs retain their TTL memory cache and remote-first disk
fallback. The model adapter reads only the Harness-owned JSON catalog at
`MODELS__URL`, with embedded configuration and environment overrides. All provider
identities use that one response format and the existing `models.json` cache.
There is no secondary OpenRouter catalog request, import, merge or fallback;
obsolete extra environment settings have no effect. Existing secondary cache files
are neither read nor removed. `server/models.json` contains curated static entries
from the public OpenAI, Anthropic, xAI and TypeSafe author namespaces on OpenRouter,
plus the original four direct-OpenAI entries. Public metadata was consulted only
at development time, including the TypeSafe Decisions catalog and verified latest
alias; catalog reads and provider connects never refresh it from OpenRouter or
invoke a model. Catalog presence/parameter support never proves account access,
inference quality or actual execution. Remote bodies, fallback files, and encoded
cache files each have a 16 MiB limit, checked before parsing or growing the encoded buffer past that
limit. Both advertised HTTP lengths and streamed chunks are bounded. Parsing,
cache encoding, memory-cache cloning, and domain conversion run on blocking
workers. Each catalog adapter owns its response/cache representation.

Cache writes are optional, private, and atomic; a failed cache write does not
invalidate a successful remote read. Invalid, unsafe, or oversized fallback files
are rejected. User-facing failures use fixed catalog context labels, and optional
cache-write warnings contain no raw errors, URLs, paths, or response content.

## Reading and caching

- The application exposes a closure-backed `ReadProviders` port. Infrastructure
  constructs the HTTP reader with a function. The `fundle` builder composes generic
  `with_memory_cache` and `with_file_fallback` wrappers around it before passing the
  port to the `read_providers` use case.
- Port clones share a memory cache, valid for five minutes by default.
- On a miss or expiry, fetch the URL (10-second timeout). Concurrent reads share
  one refresh.
- Successful responses update memory and atomically replace the disk cache.
  Persistence failures emit a tracing warning but do not discard fetched data.
- Connection, HTTP, and JSON errors fall back to the disk file regardless of age.
  Fallback results also enter memory for the configured TTL.
- If both sources fail, return an error. An empty array is a valid result.
- The default disk path is `~/.harness/providers.json`; a leading `~` is expanded
  from `HOME`. Relative configured paths are relative to the working directory.

Local verification follows the [focused-check/CI policy](ci.md), not a full local suite per edit.

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
