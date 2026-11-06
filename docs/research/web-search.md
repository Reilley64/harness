# Keyless web search for Harness

Evaluated 2026-11-06. Scope: Ketch as a per-call subprocess, SearXNG as a
self-hosted search module (including a Harness-managed subprocess), and direct
DuckDuckGo HTML search in Rust. No runtime implementation is changed by this note.

## Recommendation

Start with an explicitly configured Ketch DuckDuckGo subprocess if the goal is a
useful first release with search and page extraction. Keep a small, typed Rust
interface in front of it. Do not describe Ketch's DDG backend as more reliable than
direct DDG: they use the same upstream HTML search surface. Ketch mainly buys
integration reuse, other backends, and adjacent fetching/extraction capabilities.

If operating a search instance is acceptable, a direct HTTP adapter to an owned
SearXNG instance is the stronger long-term multi-engine option. Do not launch
SearXNG for each query. A Harness-managed long-running SearXNG child is possible,
but its lifecycle and packaging are unnecessary application responsibilities when
an external instance is available.

Choose native Rust DDG instead if avoiding another executable is more important
than outsourcing upstream parsing and extraction maintenance. Its initial HTTP
integration is small; robust HTML/challenge handling is the ongoing cost.

These are alternative implementations behind one web-search interface, not three
implementations to build immediately. Page fetching remains a separate tool.

## Evidence and limits

Sources were read from official documentation and first-party source code.
Ketch source snapshot: `833a02e3bc340bc08f76c9a5a3273d78d9368688`.
SearXNG source snapshot: `3e95f6cd2a68add0329cb162907b49289e2bc1fd`.
The locally installed Ketch binary reports `v0.19.0`; source and installed binary
are identified separately rather than assumed to be the identical build.

A temporary probe performed one search per approach with the query
`Rust tokio CancellationToken documentation`. Ketch was run with an isolated
configuration/home directory, inherited `KETCH_*` overrides removed, an explicit
`ddg` backend, no scraping, and no API credentials supplied to Ketch. The direct
request used the same query and DDG user-agent string found in Ketch's source.

| Probe               | Observation                                                                               |
| ------------------- | ----------------------------------------------------------------------------------------- |
| Ketch subprocess    | Exit 0, JSON array containing three results, approximately 0.807 s                        |
| Direct DDG HTML GET | HTTP 200, ten result-anchor markers, no detected challenge markers, approximately 0.860 s |
| SearXNG             | Documentation/source review only; no instance deployed or queried                         |

Ketch's first result was the official `tokio_util::sync::CancellationToken`
documentation. The direct HTTP probe inspected response markup, not a complete
alternative parser. These samples establish basic feasibility on this connection,
not sustained availability, search quality, relative performance, CAPTCHA rates,
or a production guarantee. No CAPTCHA bypass was attempted.

Temporary probe and response artifacts live outside the repository under
`/tmp/harness-web-search-eval/`.

## Comparison

| Dimension                   | Ketch subprocess                                                            | Owned SearXNG over HTTP                                         | Native Rust DDG HTML                                              |
| --------------------------- | --------------------------------------------------------------------------- | --------------------------------------------------------------- | ----------------------------------------------------------------- |
| Search API credentials      | Not needed with explicit DDG                                                | Not needed with selected keyless engines                        | Not needed for the evaluated HTML interface                       |
| Deployment                  | Additional executable                                                       | Python/container service and instance configuration             | No search executable/service                                      |
| Harness implementation      | Process lifecycle, bounded JSON decoding, normalization                     | Bounded HTTP/JSON client and normalization                      | HTTP client, HTML parsing, result extraction, challenge detection |
| Upstream search maintenance | Ketch maintainers                                                           | SearXNG maintainers plus instance operator updates              | Harness maintainers                                               |
| Search diversity            | Only DDG when pinned to DDG; other explicitly configured backends available | Multiple configured engines                                     | DDG only unless we implement more                                 |
| Cancellation                | Signal/terminate child, then kill and reap after a grace period             | Cancel local HTTP wait; upstream server work may continue       | Cancel local HTTP work; remote work may continue                  |
| Fetch/extraction            | Adjacent Ketch scrape capabilities                                          | Separate implementation required                                | Separate implementation required                                  |
| Privacy                     | Explicit DDG query reaches DDG from the local host's egress                 | Configured engines receive the query from the instance's egress | Same DDG exposure as Ketch/DDG                                    |

None of these choices makes search unlimited or private from the upstream engine.
Any external proxy or private instance may impose its own authentication policy.

## Ketch findings

Ketch's README documents a stateless CLI with JSON output, multiple search
backends, scrape/crawl extraction, PDF support, and browser fallback. It also
supports page caching, so 'stateless CLI' must not be interpreted as 'never writes
state'. Browser/runtime packaging should be evaluated separately before enabling
browser-backed fetching. [1]

The DDG adapter makes a GET request to
`https://html.duckduckgo.com/html/?q=...`, parses `.result` elements, extracts
`.result__title .result__a` and `.result__snippet`, and unwraps DDG redirect URLs.
It accepts HTTP 200, retries HTTP 202 up to three attempts with 500 ms sleeps,
and otherwise returns an error. [2]

This is HTML scraping, not a stronger provider contract than a native implementation.
The reviewed adapter does not explicitly distinguish a HTTP-200 page containing no
matching result nodes from a genuine empty search. Challenge/layout changes must
therefore be included in acceptance probes; a zero-result array alone cannot
prove that the query had no matches. [2]

For simple `search --json` without `--scrape`, the reviewed command emits the
result array directly. It does not include the selected auto-fallback backend in
that JSON array; the human-readable output includes a backend header. Pinning the
backend lets Harness record the chosen backend without inferring it from results.
Do not assume auto-mode JSON provides complete provider provenance. [3]

The SearXNG adapter is an HTTP client of `/search?...&format=json`, further
confirming that Ketch and SearXNG are not mutually exclusive products. If SearXNG
is the only desired backend, a native HTTP adapter avoids this extra process. [4]

The `KETCH_CONFIG` environment variable redirects configuration reads and writes.
For Harness, isolate that configuration beneath the shared app-data root and
avoid ambient provider credentials/overrides changing an explicitly keyless
setup. Pin the executable version and apply Harness-owned time/output limits. [5]

Suggested subprocess arguments, passed directly without shell interpolation:

```text
ketch search --backend ddg --limit 5 --json -- <query>
```

Search should not use `--scrape` automatically: source discovery and page reading
should remain independently bounded tool calls. Treat stderr as diagnostics, not
as a stable result schema or a safe place to expose arbitrary configuration.

## SearXNG findings

SearXNG's documented search interface is HTTP GET/POST on `/` or `/search`.
Structured output requires `format=json` and JSON enabled under `search.formats`.
Disabled formats return HTTP 403; official documentation explicitly warns that
many public instances disable them. [6]

SearXNG is a metasearch engine, not an independent web index. Its project documents
self-hosted installation options and ongoing maintenance. An owned instance makes
engine configuration and network policy explicit, but someone must operate and
update it. Configure only keyless engines if no third-party API credentials are
desired. It is not necessary for every installation to require the same optional
supporting services; avoid assuming Redis/Valkey is mandatory for all setups. [7]

The reviewed DDG engine contains DDG-specific validation-digest and bot-blocking
handling, illustrating the maintenance work that a search implementation can
accumulate beyond a first-page HTML parser. This is not proof that SearXNG always
succeeds when Ketch fails. [8]

A subprocess model has two meanings:

- Launching a search process for each query: not the normal documented SearXNG
  search interface and not recommended for Harness.
- Launching a local server once and querying it over HTTP: possible, but Harness
  then owns runtime/configuration provisioning, port allocation, readiness,
  process cleanup, upgrades, and crash recovery. It remains a service, merely
  parented by Harness.

Prefer `Harness -> owned SearXNG HTTP endpoint` over
`Harness -> Ketch -> SearXNG` when no other Ketch functionality is needed.
Public instance rotation is not a good default: availability/JSON access vary,
and it distributes queries to additional operators. [6]

SearXNG's project is AGPL-licensed; review distribution obligations if bundling or
modifying it. Ketch's repository is MIT-licensed. This note is not legal advice.
[7][9]

## Native DDG findings

Direct DDG here means consuming the public HTML search surface, not relying on a
documented JSON full-web-search contract. A native implementation can reuse
Harness's async Rust HTTP patterns, cancellation, diagnostics, and output limits.
It does not make the upstream more stable than Ketch's use of the same endpoint.
[2][8]

Beyond the happy path, Harness would own HTML parsing changes, wrapped URL
normalization, HTML entity decoding, rate-limit/challenge classification, empty
result distinction, response size/timeout limits, and any desired pagination,
locale, or safe-search behavior. Do not evade challenges, silently retry without
bounds, or map a blocked page to a successful empty result.

Its advantage is runtime/dependency control, not demonstrated superior relevance
or throughput. Search alone is relatively small; production extraction of full
HTML pages/PDFs and optional rendered pages is a separate, larger project.

## Integration seam

Expose `web_search(query, limit, ...) -> results plus provenance/diagnostics`
independently from `web_fetch(url, ...) -> extracted document`.

Keep both tool execution and subprocess/HTTP ownership in Rust. Domain values
remain Serde-free; infrastructure validates backend output and converts it to typed
results. Return actual backend/instance provenance, bounded titles/URLs/snippets,
and explicit failures/truncation. Never expose provider credentials or treat
retrieved content as trusted instructions.

The application must enforce deadlines, query/result/output limits, cancellation,
and approved destinations. An explicitly configured private SearXNG endpoint is
a narrow exception to public web-fetch destination rules, not permission for the
model to fetch arbitrary private-network URLs.

## Sources

1. [Ketch README](https://github.com/1broseidon/ketch/blob/833a02e3bc340bc08f76c9a5a3273d78d9368688/README.md)
2. [Ketch DDG adapter](https://github.com/1broseidon/ketch/blob/833a02e3bc340bc08f76c9a5a3273d78d9368688/search/ddg.go)
3. [Ketch search command and output](https://github.com/1broseidon/ketch/blob/833a02e3bc340bc08f76c9a5a3273d78d9368688/cmd/search.go)
4. [Ketch SearXNG adapter](https://github.com/1broseidon/ketch/blob/833a02e3bc340bc08f76c9a5a3273d78d9368688/search/searxng.go)
5. [Ketch configuration path](https://github.com/1broseidon/ketch/blob/833a02e3bc340bc08f76c9a5a3273d78d9368688/config/config.go)
6. [SearXNG search API](https://docs.searxng.org/dev/search_api.html)
7. [SearXNG installation](https://docs.searxng.org/admin/installation.html) and [project README/license](https://github.com/searxng/searxng/blob/3e95f6cd2a68add0329cb162907b49289e2bc1fd/README.rst)
8. [SearXNG DDG engine](https://github.com/searxng/searxng/blob/3e95f6cd2a68add0329cb162907b49289e2bc1fd/searx/engines/duckduckgo.py)
9. [Ketch license](https://github.com/1broseidon/ketch/blob/833a02e3bc340bc08f76c9a5a3273d78d9368688/LICENSE)
