# GitHub automation

This guide owns CI/CD terminology and YAML layout under `.github/`. The rules
in the root `AGENTS.md` still apply.

## CDF naming and grouping

Use the CD Foundation's
[Pipeline Terminology](https://github.com/cdfoundation/sig-interoperability/blob/main/docs/pipelines-terminology.md)
as reference terminology. It is unfinished guidance, not a normative standard.
The prefixes and formatting below are project conventions.

Workflow and job display names use `Stage: Description`. Choose stages by the
work performed:

| Stage | Meaning |
| --- | --- |
| Build | Compile or assemble software, documentation or generated configuration. |
| Test | Test, scan or verify software and project rules. |
| Release | Package, version, sign or publish artifacts and release records. |
| Deploy | Change an external environment and verify deployment. |
| Operations | Monitor deployed assets. |

Step display names use `Type: Description`. Use these distinctions:

- Source retrieves repository contents.
- Setup prepares tools, dependencies, caches, credentials or workspace inputs.
- Build compiles software or renders configuration.
- Test runs a test suite. Scan performs non-test verification, including linting.
- Policy enforces project rules, including pull request title requirements.
- Package creates a distributable artifact. Tag applies a version label;
  computing a tag string alone is Setup. Publish uploads artifacts or records.
- Deploy applies infrastructure or updates service configuration.
  Verify Deployment checks the rollout.
- Run covers automation spanning several types, such as release-please's
  release requests, tags and publication. Release is a stage, not a step type.

Use combined prefixes when one action spans categories, such as
`Build and release` for a job or `Build, package and publish` for an image action.
Separate jobs when execution gates or permissions differ. Separate Setup and
Deploy commands into steps when they share a runner and working directory.
Keep dependency order driven by execution needs, not the stage list above.
For additional step types, consult the linked CDF definitions.

## YAML layout

- Use two-space indentation and double-quote display names containing `:`.
- Order workflow blocks as name, triggers (`on`), permissions, concurrency,
  environment and jobs. Separate top-level blocks and jobs with one blank line.
- Within a job, group name, dependencies (`needs`), condition (`if`), runner
  and timeout first. Follow with permissions, outputs, environment/defaults,
  then steps. Separate these groups with one blank line; omit absent groups.
- Begin steps with name, then optional id and condition. Keep action `uses`
  with its `with` inputs. Put shell, working directory and environment before
  a shell step's `run`. Preserve other keys beside the behavior they configure.
- Separate steps with one blank line. Keep an explanatory comment directly
  above its step or setting, without a blank line between them.
- Expand multiple job dependencies as a block sequence. Short trigger filters
  may use inline lists, such as `branches: [main]`.
- Use `|` for multi-command scripts and `>-` for a single folded command.
  Keep commands in execution order and preserve expression and quoting semantics.

## Project tasks

Keep shared build, test, lint and format orchestration in `mise.toml`. Workflows
install tools with the Mise action and invoke tasks rather than duplicating Cargo
or Bun commands. Before changing those tasks or CI coverage, read
[CI notes](../docs/ci.md).

## Changes and verification

When changing job display names, check GitHub required-check configuration for
old contexts. Report unavailable remote checks rather than assuming they are clear.

For formatting-only edits, compare parsed YAML before and after with a parser
that preserves GitHub's `on` key and expression strings. Completion requires
identical values and sequence order. For structural edits, verify every `needs`
and output reference, event condition, permission change and deployment gate.

Run `git diff --check` and Actionlint on the changed workflows. Invoke Actionlint
through the installed CLI or `go run github.com/rhysd/actionlint/cmd/actionlint@v1.7.12`.
If runner labels or action metadata produce diagnostics, verify them against the
runner configuration or upstream action definition before using narrow filters;
report filtered diagnostics and any disabled shell checks. Report commands that
failed or were unavailable.
