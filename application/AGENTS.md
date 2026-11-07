# Application

Follow [the repository coding standards](../CODING_STANDARDS.md).

## Use-case modules

- Keep each use case's `call` and orchestration in `src/use_cases/<name>.rs`.
- Declare private `dependencies`, `error`, and `output` child modules, each in
  its own file under `src/use_cases/<name>/`.
- Define `Dependencies`, `Error`, and `Output` in their respective child modules
  and re-export them from the use-case file.
- Apply this layout to small use cases too; keep the type definitions in the
  child modules rather than inline in the use-case file.

## Turn coordination

- Keep pure state rules in domain transition methods. Use cases own effect
  ordering, cancellation, budgets, and checkpoints rather than reimplementing
  tool/turn transition validation.
- Preserve checkpoint-before-effect and result-before-continuation ordering.
  Acknowledge steering only after its required durable checkpoint; idempotent
  instructions must not repeat actions.
- Account for generation attempts durably before inference. Retain reported usage
  from superseded/cancelled generations without retaining their discarded
  continuation. Missing reports remain unknown, not zero.
- Recover interrupted work conservatively; resume does not replay uncertain tool
  executions. Keep finalization and steering acceptance under backend authority.

Before changing lifecycle, persistence ordering, steering, or accounting, read
[conversations](../docs/conversations.md). Before changing execution policy or
budgets, read [tool calls](../docs/tool-calls.md). Preserve the existing
closure-backed port/use-case interfaces unless the requirement changes them.
