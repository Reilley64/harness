# TUI

Follow [the repository coding standards](../../CODING_STANDARDS.md).

## Ownership

- Components render and compose atomic UI. Hooks own asynchronous UI lifecycles;
  services own ACP transport and display-only I/O. Shared presentation types stay
  independent of React and transport.
- Rust owns durable turns, tool execution, and steering. Submit steering to the
  backend rather than implementing cancellation-and-resend in React.
- Bind display-only metadata, including Git branch lookup, to the conversation's
  workspace. Preserve the invoking directory through launch/build scripts;
  explicit resume retains the saved workspace.

## Interaction and rendering

- Preserve submitted whitespace except CRLF/CR normalization. Use shared prompt
  geometry for rendering and navigation: terminal cells and whole graphemes, not
  string length. Clip the viewport, not the draft.
- Enter submits or steers; Escape closes or back-navigates while preserving drafts.
  Use explicit scoped Stop; Ctrl+X in the editor remains cut. Retain
  Alt+Enter/Ctrl+J newline fallbacks for terminals without Shift+Enter.
- Keep terminal modes and asynchronous resources paired with cleanup. Verify
  startup/input changes in a real PTY; component snapshots cannot detect echoed
  capability replies or raw-mode leaks.
- Keep transcript work out of animation ticks. Share color clocks among leaf
  subscribers and preserve stable layout caches and scroll anchors.

Before changing prompt input, transcript layout, tool panels, theme, status rows,
footer, scrolling, or terminal startup, read [frontend](../../docs/frontend.md).
When changing usage display, also read [conversations](../../docs/conversations.md).
Visual details and limits belong in those documents, not duplicate constants here.
