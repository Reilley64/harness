import type { DOMElement } from "ink";
import { Box, useBoxMetrics, useInput } from "ink";
import { useCallback, useMemo, useRef } from "react";
import { useTerminalScroll } from "../../hooks/use-terminal-scroll.js";
import { transcriptLines } from "../../services/transcript.js";
import type { Message } from "../../types/conversation.js";
import type { TranscriptViewport } from "../../types/transcript-viewport.js";
import { MessageLine } from "../molecules/message-line.js";

type Props = {
  messages: Message[];
  columns: number;
  rows: number;
  focused: boolean;
  viewportState: TranscriptViewport;
  changeViewport: (
    update: (current: TranscriptViewport) => TranscriptViewport,
  ) => void;
};

export function Transcript({
  messages,
  columns,
  rows,
  focused,
  viewportState,
  changeViewport,
}: Props) {
  const { anchor, expandedTools } = viewportState;
  const viewport = useRef<DOMElement>(null);
  const metrics = useBoxMetrics(viewport);
  const height = Math.max(
    1,
    metrics.hasMeasured ? metrics.clientHeight : rows - 6,
  );
  const lines = useMemo(
    () => transcriptLines(messages, columns, expandedTools),
    [messages, columns, expandedTools],
  );
  const locate = (current: typeof anchor) => {
    if (!current) return lines.length;
    const index = lines.findIndex((line) => line.id === current.id);
    return Math.min(lines.length, index >= 0 ? index + 1 : current.end);
  };
  const end = locate(anchor);
  const visible = lines.slice(Math.max(0, end - height), end);
  const scrollBy = useCallback(
    (rows: number) => {
      changeViewport((state) => {
        const current = state.anchor;
        let currentEnd = lines.length;
        if (current) {
          const index = lines.findIndex((line) => line.id === current.id);
          currentEnd = Math.min(
            lines.length,
            index >= 0 ? index + 1 : current.end,
          );
        }
        const next = Math.max(
          Math.min(height, lines.length),
          Math.min(lines.length, currentEnd + rows),
        );
        return {
          ...state,
          anchor:
            next >= lines.length
              ? null
              : { id: lines[next - 1]?.id ?? "", end: next },
        };
      });
    },
    [lines, height, changeViewport],
  );
  useTerminalScroll(scrollBy, focused);

  useInput(
    (_input, key) => {
      if (key.eventType === "release") return;
      if (key.ctrl && _input === "o")
        changeViewport((current) => ({
          ...current,
          expandedTools: !current.expandedTools,
        }));
      else if (key.pageUp) scrollBy(-Math.max(1, height - 1));
      else if (key.pageDown) scrollBy(Math.max(1, height - 1));
      else if (key.shift && key.upArrow) scrollBy(-1);
      else if (key.shift && key.downArrow) scrollBy(1);
    },
    { isActive: focused },
  );

  return (
    <Box
      flexDirection="column"
      ref={viewport}
      flexGrow={1}
      flexShrink={1}
      minHeight={0}
      justifyContent="flex-end"
      marginX={-1}
      overflow="hidden"
    >
      {visible.map((line) => (
        <MessageLine key={line.id} line={line} />
      ))}
    </Box>
  );
}
