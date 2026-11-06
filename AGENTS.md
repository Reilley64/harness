# Harness

Read [CODING_STANDARDS.md](CODING_STANDARDS.md) before editing. It owns the shared
coding and verification rules; scoped instructions add layer-specific guidance.

Read the applicable instructions before changing a layer:

- Domain: [domain/AGENTS.md](domain/AGENTS.md).
- Application: [application/AGENTS.md](application/AGENTS.md).
- Infrastructure: [infrastructure/AGENTS.md](infrastructure/AGENTS.md).
- ACP: [presentation/acp/AGENTS.md](presentation/acp/AGENTS.md).
- TUI: [presentation/tui/AGENTS.md](presentation/tui/AGENTS.md).
- GitHub workflows and automation: [.github/AGENTS.md](.github/AGENTS.md).

When changing domain terminology, read [GLOSSARY.md](GLOSSARY.md). When changing
behavior, update its owning document under `docs/`; keep detailed product behavior
there rather than copying it into agent instructions. Keep Harness-specific rules
in this repository, not personal/global agent configuration.

## Agent skills

### Issue tracker

Issues and specs live in Plane Cloud in the `harness` project. Resolve workspace and project IDs from local Git config. See `docs/agents/issue-tracker.md`.

### Triage labels

The five triage roles are Plane states, not labels. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `GLOSSARY.md`; ADRs are Plane pages under Architecture decisions. See `docs/agents/domain.md`.
