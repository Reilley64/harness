# Public-seam verification

Use red -> green at pre-agreed public seams, one vertical behavior at a time. Write the expected user-visible result before implementation. Prefer actual orchestration/adapters and disposable external-boundary fixtures; label substitutions and evidence limits.

Observe through public application/protocol/provider/persistence/native paths rather than private helpers or debug APIs. Keep verification independent from implementation calculations. Follow repository policy for test location/lifetime; for Harness retain temporary external probe sources/artifacts outside removable worktrees, adding no permanent tests or test-only dependencies.

Run the repository's existing documented build/test/check commands and diff checks. Report failed/missing prerequisites rather than silently installing tools, suppressing checks or calling fixture-green live quality. When interface shape is uncertain, use reference/deep-modules.
