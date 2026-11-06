# Domain

Follow [the repository coding standards](../CODING_STANDARDS.md).

- Declare modules and re-export domain types from `src/lib.rs`.
- Own pure invariants and state transitions here: active replies, tool lifecycle,
  identities, and steering acceptance. Coordinators call these methods instead
  of duplicating their rules.
- Keep runtime stopping distinct from crash recovery; they are different policies,
  not interchangeable implementations of one transition.

Before changing turn/tool transitions or recovery, read
[conversations](../docs/conversations.md) and [tool calls](../docs/tool-calls.md).
Keep serialized record validation and representation conversion in the owning
infrastructure adapters.
