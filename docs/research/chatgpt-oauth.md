# ChatGPT subscription authentication

## Primary-source findings

OpenAI documents Sign in with ChatGPT plan usage for open-source and locally
hosted applications. Eligible subscription access is distinct from API-key
billing; paid or remotely hosted integrations have a separate interest process.
Source: https://developers.openai.com/siwc/token-sharing-open-source

The documented first-time registration uses `client_id=dynamic_agent_client`,
`agent_name_hint` set to the application's actual name, and a stable per-host
`ext_agent_host_id`. It does not require a client secret or partner API key.
The callback supplies the issued client ID; that ID, not the registration
entrypoint, is used for code exchange and subsequent sign-ins.
Source: https://developers.openai.com/siwc/token-sharing-open-source/sign-in

Authorization uses `https://auth.openai.com/api/accounts/authorize`. Code exchange
uses `https://auth.openai.com/api/accounts/oauth/token`. Both use
`resource=https://api.openai.com/v1`. Requested scopes are `openid profile email
offline_access resource.invoke chatgpt.tokens.use.direct`.
Source: https://developers.openai.com/siwc/token-sharing-open-source/sign-in

Clients must start a loopback listener before opening the browser, use
`http://127.0.0.1:<port>/auth/callback` consistently, and generate fresh state,
nonce, and S256 PKCE per attempt. Validate state, the ID-token signature,
issuer, issued-client audience, expiry, and nonce. Check granted plan-usage
scopes; a valid identity token alone does not authorize inference.
Source: https://developers.openai.com/siwc/token-sharing-open-source/sign-in

Persist host identity across restarts. Keep issued-client/account mappings
separate and confirm identity on returning sign-in. Store credentials atomically
with owner-only permissions and never log them. Refresh tokens rotate and
refreshes must be serialized; sign-out uses the discovery document's revocation
endpoint. Retained ID-token authorization hints must be redacted from logs.
Sources:
- https://developers.openai.com/siwc/token-sharing-open-source
- https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions
- https://developers.openai.com/siwc/token-sharing-open-source/token-reference

Future inference through this flow uses the Responses API, `store: false` and
`stream: true`, with the history included in the input. Supported parameters and
tools are limited during preview; a subscription does not imply unrestricted API
access or a separate usage allowance for Harness.
Sources:
- https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference
- https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations
- https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions

## Harness design

Authentication runs in Rust, behind a closure-backed application port. An
infrastructure factory matches provider IDs and selects provider-specific
implementations. The ACP adapter presents authentication and forwards a browser
challenge; Ink displays progress, the authorization URL, cancellation, and the
result. Tokens never cross ACP.

Use Harness's documented dynamic-registration flow, not Codex's public OAuth
client ID or its internal device-auth endpoints. No Codex installation is needed.
Do not send an ID-token hint to Ink as part of the authorization URL; returning
sign-in can use the issued client ID and the browser's account selector instead.

The initial implementation covers sign-in, returning sign-in, validation,
credential persistence, timeout, and cancellation. Chat, account switching,
logout/revocation, and token consumption/refresh for inference are separate work.
Persist the refresh token and expiry now; do not create an unused background
refresh loop before there is a model-request consumer.
