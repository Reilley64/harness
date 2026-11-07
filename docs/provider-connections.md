# Provider connections

## Scope

The `create_provider_connection` use case signs in and saves one active
connection per provider. The initial provider is ChatGPT (`openai`). Repeating
sign-in reuses the active registration and stable connection ID, verifies the
same account, and replaces credentials only after validation.

Text chat consumes this connection and refreshes credentials on demand near
access-token expiry. Account switching, connection listing, and logout/revocation
are not implemented. No background refresh loop runs.

Successful setup initializes an uncustomized provider wildcard through the application
policy port. Re-authentication never broadens a custom model allowlist, including
an empty list. Legacy credential discovery alone grants no model coverage. See
[model allowlist](model-allowlist.md) for settings, initialization and commit
boundaries.

## Modules and interfaces

- `domain/src/provider_connection.rs`: safe connection identity and account
  display information, without Serde or tokens.
- `application/src/use_cases/create_provider_connection/`: `Dependencies`,
  `Output`, and use-case `Error`. `call` takes dependencies, provider ID, and the
  authorization-presentation callback directly.
- `application/src/ports/`: one closure-backed port per file, re-exported through
  `mod.rs`. `create_provider_connection_factory.rs` selects a creation port by
  `ProviderId`; `create_provider_connection.rs` owns that provider-bound port;
  `present_authorization.rs` owns its per-operation authorization-presentation
  callback and challenge. A private `port!` macro in `mod.rs` generates each
  port's `Arc` storage, `Clone`, constructor, and forwarding `call` method;
  associated errors remain explicit.
- `infrastructure/src/create_provider_connection_factory/mod.rs`: factory whose provider-ID match
  selects OpenAI or returns an unsupported-provider error.
- `infrastructure/src/create_provider_connection/openai.rs`: the documented ChatGPT flow,
  discovery, validation, and returning-account checks.
- `infrastructure/src/create_provider_connection/callback.rs`: per-attempt loopback listener.
- `infrastructure/src/create_provider_connection/storage.rs`: private, atomic credential
  storage and a process-level file lock.
- `infrastructure/src/dependencies.rs`: `fundle` composition, error wrapping,
  and conversion of infrastructure DTOs into domain connections.
- `presentation/acp/`: standard `authenticate` mapped onto the use case, with
  a custom browser-challenge notification and safe response metadata.
- `presentation/tui/`: picker, progress, cancellation, manual browser reopening,
  and result display. No token handling.

The use case first calls `CreateProviderConnectionFactory` with the requested
`ProviderId`. Factory selection is synchronous and fallible; unsupported
providers fail before authorization starts. It then calls the returned
`CreateProviderConnection` with the authorization-presentation callback.
The creation port is already bound to its provider and does not take a provider ID.

The operation is a single future rather than a start/poll/complete registry.
Its callback presents the browser challenge while the future waits for the
browser callback. ACP runs it as connection-owned concurrent work so cancellation
is processed promptly.

## ChatGPT flow

The implementation follows OpenAI's
[open-source/local-app Sign in with ChatGPT documentation](https://developers.openai.com/siwc/token-sharing-open-source/sign-in).
It does not copy Codex's OAuth client ID or use its internal device-auth endpoints.

1. Acquire the provider credential lock, load credentials, and persist a new
   opaque host ID if needed.
2. Bind an ephemeral port on `127.0.0.1` with the fixed `/auth/callback` path.
3. Generate fresh 256-bit state and nonce values plus S256 PKCE.
4. Authorize at `https://auth.openai.com/api/accounts/authorize`. New registrations
   use `dynamic_agent_client` and `agent_name_hint=Harness`; returning sign-in
   uses the saved issued client ID. Every attempt sends the stable host ID.
5. Validate callback state using constant-time comparison. Incorrect state does
   not end the attempt. Valid provider rejection ends it without code exchange.
6. Require an issued client ID on initial registration; reject a changed client
   ID on returning sign-in.
7. Exchange the code at `/api/accounts/oauth/token`, using the same loopback URI,
   PKCE verifier, issued client ID, and `https://api.openai.com/v1` resource.
8. Require Bearer access and refresh tokens and the granted `openid`,
   `resource.invoke`, and `chatgpt.tokens.use.direct` scopes.
9. Use OpenID Connect discovery and published signing keys to verify the ID
   token's signature, issuer, audience, expiry, and nonce. Verify `at_hash` when
   present. On returning sign-in, require the same verified subject.
10. Atomically save the registration and tokens, then return safe connection
    information. Failed attempts do not replace previously saved credentials.

Account eligibility and preview limitations still apply. This does not grant
unrestricted developer-API access or an independent usage allowance. Text
inference follows the documented Responses API requirements.

## Credential storage

The default directory is `~/.harness/auth/`; OpenAI uses `openai.json` and a
separate `openai.lock`. Both paths derive from `app_data_path/auth`; the single
storage-root override is `APP_DATA_PATH`. Other settings use the existing
unprefixed, double-underscore environment scheme; no runtime TOML file is loaded.

The credential record stores the persistent host ID, active issued client ID,
and registrations keyed by issued client ID. Each registration retains its
connection ID, issuer, verified subject, token set, save time,
and access-token expiry. Secret records deliberately do not derive Debug.

On Unix, directories are created with mode 0700 and files with mode 0600.
An existing insecure directory/file is rejected rather than silently changing
permissions on a user-selected location. Credentials are serialized, written to
an owner-only temporary file in the same directory, flushed and synced, then
atomically renamed, then the directory is synced. The private replacement helper
is shared with conversation storage. Credential locking and no-follow bounded
file reads are shared technical helpers; serialized records remain adapter-owned.
The opened file must be regular, owned by the current user, owner-only, and have
one hard link. Token, refresh, discovery and verification-key response bodies
are bounded at 1 MiB, including chunked responses. Identity endpoints require
HTTPS without URL credentials, except loopback HTTP fixtures. Reads/writes use Tokio; temporary creation,
directory sync, and file locking run under `spawn_blocking`.

The lock spans each sign-in attempt and protects against concurrent writers
across processes. Concurrent attempts fail rather than queue browser flows.
Corrupt, oversized, or unsupported credential files fail without being replaced.
Storage failure is an authentication failure, not a warning.

File-backed secret protection is currently implemented only on Unix. Other
platforms fail explicitly; Windows needs an ACL-protected or native credential
store before sign-in is enabled there.

## Cancellation and limits

The backend timeout defaults to 600 seconds; zero is invalid. HTTP requests have
a 15-second timeout and do not follow redirects. The frontend has a 660-second
safety timeout, so lowering the backend timeout works without frontend changes.

Cancel, timeout, or disconnect drops the pending operation, closes the callback
listener, and releases the lock. Cancellation does not revoke an authorization
already granted at OpenAI; logout/revocation remains separate work. A request
cancelled at the persistence commit point may already have saved a completed
connection.

The authorization URL contains no ID-token hint, access token, or refresh token.
Only the opaque local connection ID and provider ID cross ACP on success;
account subjects remain private to credential adapters. Display labels are not
stored separately; legacy label fields are ignored and removed on the next
credential save. The TUI hides the temporary
authorization URL until explicitly revealed with `l`. Do not log or commit credential files,
and use the production issuer except in a trusted environment. Plain HTTP issuer
overrides are accepted only for loopback fixtures.

## On-demand refresh

The provider-bound reply adapter resolves the saved connection under the same
cross-process credential lock used by sign-in. It reloads after acquiring the
lock, so concurrent conversations/processes use the latest rotated token set.
Unlike sign-in, a reply waits for a busy credential lock and can cancel that wait.

Refresh starts within 60 seconds of expiry, honoring `earliest_refresh_at` when
present (Unix seconds or RFC 3339). The token endpoint receives
`grant_type=refresh_token`, the saved issued client ID, refresh token, and resource;
scope is omitted to retain the grant. Replacement Bearer tokens, expiry, and
required scopes are validated. A returned ID token is verified for signature,
issuer, audience, expiry, stable subject, and optional access-token hash.

The replacement is atomically saved before using its access token for inference.
Once a refresh exchange begins, cancellation waits for verification/persistence
instead of dropping a rotating replacement. Individual credential HTTP requests
have a 15-second timeout and redirects remain disabled. Inference itself is
cancellable, has its own timeout, and is never retried after output begins.
Refresh or identity failure does not start inference; sign in again if necessary.
See [conversations](conversations.md) for cancellation and storage boundaries.

## Validation

Temporary probes outside the repository exercise:

- Dynamic and returning registration, stable host/client/connection IDs, fresh
  state/nonce/PKCE, exact code-exchange redirect/resource, and private storage.
- Invalid state, signature, issuer, audience, expiry, nonce, scopes, access-token
  hash, changed client ID, and changed returning account.
- Preservation of existing credentials on failure.
- Concurrent-attempt rejection, timeout, cancellation, and listener/lock cleanup.
- Official TypeScript SDK requests/notifications, AbortSignal cancellation, Ink
  picker/progress/success/error views, command completion, and clean terminal
  shutdown, including quitting during authorization.
- Absence of access and refresh tokens from ACP messages and terminal output.

These are local signed-token fixtures, not a completed login against a real
ChatGPT account. A real browser sign-in remains a manual verification step.

Further primary sources are collected in
[the OAuth research notes](research/chatgpt-oauth.md).
