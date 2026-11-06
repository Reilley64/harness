# Continuous integration

GitHub automation follows [.github/AGENTS.md](../.github/AGENTS.md).

## Shared local and CI tasks

Install tools with `mise install`. `mise.toml` owns every project tool pin,
including Rust's nightly version, minimal profile, and Clippy/rustfmt components.
Use Mise tasks or `mise exec -- cargo …` so ad hoc Cargo commands use the pinned
toolchain too; there is no separate Rustup toolchain file.

| Command | Purpose |
| --- | --- |
| `mise run` | Install TUI dependencies, build the ACP agent, and launch the TUI. |
| `mise run build` | Build the debug Rust workspace and all targets. |
| `mise run build:release` | Build the release workspace and all targets. |
| `mise run test` | Run all Rust test targets and doctests. |
| `mise run lint` | Run strict Clippy, Biome lint/import checks, Actionlint, ShellCheck and redacted secret scanning. |
| `mise run format` | Format Rust and the TUI. |
| `mise run typecheck` | Check TUI types. |
| `mise run check` | Check formatting, lint, types and automation without rewriting source. |

Rust/ACP build and start commands are owned by Mise, not `package.json`. TUI-local
check/format scripts stay package-local and are called by Mise. `bin/harness`
delegates to the Mise launch task while preserving the invoking workspace.

## Workflows

Both workflows run on pushes, pull requests, and manual dispatch. All jobs
currently run on Linux (`ubuntu-latest`). Superseded runs of the same workflow/ref
are cancelled. Jobs use read-only repository permissions and do not receive
provider credentials or require a catalog server.

- [Test: Workspace](../.github/workflows/test.yml) uses the Mise action, then runs
  `check:rust` and `test`, `check:tui`, and automation/secret lint in separate jobs.
- [Build: Rust workspace](../.github/workflows/build.yml) uses the Mise action,
  then runs `build:release`. It does not publish artifacts, releases or deployments.

Rust dependency/build caches are separated between test and release-build jobs.
External actions, including policy checkout, are pinned to commit SHAs; checkout
does not persist repository credentials. Keep `Cargo.lock` and
`presentation/tui/bun.lock` in Git: dependency installation and Cargo build/test
commands refuse to regenerate lockfiles.

## Secret checks

`mise run lint:secrets` uses pinned Gitleaks with full match redaction, a timeout,
and no inline allow-comment bypass. It scans current first-party files, including
untracked files; Git internals, build outputs and third-party dependencies are
excluded by `.gitleaks.toml`. No report containing secret values is uploaded.
This check is included in `lint`, `check`, and Linux CI. Pattern scanning cannot
prove the absence of arbitrary secrets or personal data; see
[security and privacy](security.md).

## Test coverage

There are currently no checked-in Rust tests or Bun test suite. Cargo's test job
is ready to discover Rust tests and doctests, but passing zero tests is not
behavioral coverage. Per project policy, temporary external regression probes
remain outside the repository and are not available to GitHub runners.

The frontend runs TSX directly with Bun, so there is no separate frontend bundle
build. Its CI checks are dependency installation, typechecking, and Biome.

## Pull request policy

`test-policy.yml` checks pull request titles and new explanatory source comments.
Its existing read-only permissions, pull request events, and check names remain
unchanged. The comment job checks out full history and runs the executable,
dependency-free Python 3 checker:

```sh
scripts/check-no-new-comments origin/main
```

Pass the actual base ref rather than assuming `main`. The checker resolves that
ref to a commit, finds its merge base with `HEAD`, and inspects committed changes
to `HEAD`, not the index or working tree. A missing ref, unborn `HEAD`, unreadable
Git object, decoding failure, or tokenizer failure exits 2 with a generic safe
diagnostic. Exit 1 means policy violations; exit 0 means none. Diagnostics contain
JSON-escaped filenames and line numbers, never comment/source text. Paths and
refs are passed directly to Git without shell interpolation; NUL-delimited path
lists and literal pathspecs preserve spaces, newlines, and special characters.
Git-detected renames compare against their original paths (with a low similarity
threshold), so unchanged comments do not become new merely by moving a file or
editing adjacent code on the same line.

The checker recognizes Rust, TS/JS (including TSX/JSX and module extensions),
shell, Python, TOML and YAML, plus extensionless Python/shell/Node/Bun/Deno scripts
with a shebang. Python uses its standard tokenizer; the other languages use lexical
state for quoted/raw/template strings, regex literals, inline/block comments,
shell heredocs and YAML block scalars. New lines inside existing block comments
are checked too. Baseline comment fragments are matched against untouched lines
first, then against added diff lines: editing code beside an existing comment is
allowed, but inserting a duplicate comment is still new. This is a
lexical policy check, not a compiler or full language parser; unusual embedded
languages and context-dependent syntax still require review.

Shebangs, Python encoding declarations, TS compiler directives (`@ts-check`,
`@ts-nocheck`, `@ts-ignore`, `@ts-expect-error`), triple-slash references, and JSX
compiler pragmas are allowed in their directive forms, not with appended prose.
Rust attributes are code, not comments. Markdown/reStructuredText, `docs/`, known
dependency/build/generated directories, named generated/minified files, lockfiles,
symlinks and binary blobs are excluded. Exclusions are path-based; keep generated
outputs in those locations rather than relying on a generated-file comment.

For a read-only syntax check without bytecode artifacts:

```sh
python3 -c 'import ast, pathlib; ast.parse(pathlib.Path("scripts/check-no-new-comments").read_text())'
```

The policy comparison requires a valid base and committed `HEAD`; syntax checking
can run in ordinary local lint even before the first commit. Temporary external
Git fixtures exercise policy behavior; no permanent test suite or parser
dependency is installed.
