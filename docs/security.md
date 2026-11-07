# Security and privacy boundaries

## What leaves the machine

The configured model provider receives submitted messages, accepted steering,
retained model continuation, tool definitions, and recorded tool results needed
to continue the conversation. Responses requests use `store: false`; that is a
provider API setting, not a guarantee about the provider's retention practices.
Web search sends its query to DuckDuckGo. Catalog endpoints receive catalog GETs.
OAuth authorization, token exchange and identity verification contact the issuer.
Provider URLs require HTTPS, with loopback HTTP allowed for fixtures; credential
requests and inference do not follow redirects. Token, refresh, discovery and
verification-key response bodies are bounded at 1 MiB.

Harness does not proactively insert OAuth credentials, account emails, OIDC
subjects, or the absolute workspace path into model prompts. Relative tool paths
are resolved against the saved canonical workspace in Rust. Tool results can
still reveal paths and personal content. A model instruction discouraging secret
access is guidance, not an access-control boundary.

## Local presentation and diagnostics

ACP authentication publishes only the opaque local connection ID and provider
ID. Account subjects remain private to credential adapters for identity binding;
no account identity is exposed through application connection outputs. Account
display labels are no longer stored separately; legacy labels are ignored and
removed on the next credential save. Access, refresh and ID
tokens, private provider continuation, encrypted reasoning and provider call IDs
are excluded from public conversation/tool snapshots. Authorization challenges
necessarily carry a temporary browser URL to the local ACP client; the TUI hides
it by default. `o` opens the browser; `l` explicitly reveals/hides the URL for
manual sign-in. Revealing it can disclose temporary identifiers in screenshots.
The footer intentionally displays the workspace and branch requested by the user.

Diagnostics publish explicit static error contexts. Arbitrary underlying error
messages fall back to a generic message; retained causes are not automatically
printed. The TUI drains and discards raw agent stderr rather than displaying it.
Startup errors use the same safe boundary. Rust panics print only a static
failure message. The TUI distinguishes explicit presentation errors from unknown
JavaScript/SDK errors; unknown errors are not stringified. Only managed-Rust
application error codes carry the backend's safe diagnostic strings. Cache errors do not log raw URLs,
paths, response bodies or causes. Tool stdout and stderr are task data, not safe
diagnostics: their content follows the tool-result boundary below.

## Storage and subprocesses

Credentials, conversations, preferences and artifacts are local plaintext, not
encrypted storage. Unix app-data directories must be real, owned by the current
user and owner-only. Credential/cache reads validate the opened file, reject
symlinks, non-regular files, hard links and non-owner permissions, and enforce a
read bound. Shared credential locks use no-follow opens and blocking-worker lock
acquisition. Replacements use private temporary files and atomic replacement.
These controls do not protect against root, the same OS user, compromised process
memory, or every filesystem/ACL configuration.

Tool subprocesses receive a small environment allowlist: executable/home/runtime
paths, locale, terminal, temporary-directory and Rust toolchain settings. Provider
and cloud credentials, arbitrary project variables and Harness configuration are
not inherited. Tasks that need additional environment values must set them
explicitly. This does not prevent commands from reading files or other resources
available to the same OS user.

## Repository checks and limits

`mise run lint:secrets` runs pinned Gitleaks rules over first-party files, including
untracked files, with matched secrets fully redacted from scanner output. Generated
build trees, Git internals and third-party dependencies are excluded. The scan is
part of local `lint`/`check` and Linux CI; inline allow-comments do not bypass it.
It is pattern detection, not proof that arbitrary secrets or personal information
are absent. It scans the current tree, not Git history. If a real credential was
committed or disclosed, revoke/rotate it; deleting the current file is insufficient.
Git ignore rules cover common local environment files, key material and default
in-repository runtime-storage paths, but cannot protect every custom location.

Ordinary tools remain automatic, unsandboxed and run with OS-user permissions.
The [Setup output workflow](repository-setup.md) stages supported file writes as
preserved-content drafts; only a trusted local ACP/native confirmation can apply
that exact proposal. This is not a sandbox for trusted bash or future MCP tools,
nor consent for machine-wide installs, external services or authentication. File reads,
bash commands and web searches can expose secrets or personal information through
outputs or queries; those outputs may be saved locally and sent to the configured
model provider. There is no reliable universal redaction or confidentiality
claim for arbitrary tool content. Use an isolated OS account/container and avoid
mounting credentials or sensitive data when a task needs a stronger boundary.
