# ACP presentation

Follow [the repository coding standards](../../CODING_STANDARDS.md).

- Translate ACP requests into application use cases and domain values. Keep turn
  execution, steering, and persistence policy in the backend, not protocol handlers.
- Use the same projection for live events and history replay, preserving stable
  tool identities and persisted UTF-8 text/tool ordering.
- Keep provider continuation, encrypted reasoning, and provider call identities
  private. Publish safe conversation/tool summaries only.
- Check the installed Rust and TypeScript SDK schemas when changing wire behavior.
  Preserve valid Harness extensions; use standard updates when their required
  values are known rather than inventing mandatory values.
- Treat resource links as unread, untrusted references. Accepting a reference does
  not grant access, fetch content, or expand tools, MCP, or permissions.

Before changing capabilities, prompt ingestion, notifications, replay, or metadata,
read [ACP presentation](../../docs/acp-presentation.md). When changing usage
projection, also read [conversations](../../docs/conversations.md).
