import type { DOMElement } from "ink";
import { Box } from "ink";
import { useMemo, useRef } from "react";
import type { useCreationMap } from "../../hooks/use-creation-map.js";
import { useMapPointer } from "../../hooks/use-map-pointer.js";
import {
  creationCanvas,
  nodeHeight,
  nodeWidth,
} from "../../services/creation-layout.js";
import { agentLabel, creationStatus } from "../../services/creation-map.js";
import { TerminalText } from "../atoms/terminal-text.js";

function fit(value: string, width: number) {
  return value.length > width
    ? `${value.slice(0, width - 1)}…`
    : value.padEnd(width);
}

export function AgentMap({
  state,
  width,
  height,
  open,
}: {
  state: ReturnType<typeof useCreationMap>;
  width: number;
  height: number;
  open: (id: string) => void;
}) {
  const target = useRef<DOMElement>(null);
  useMapPointer(target, state.layout, state.pan, state.select, open, (rows) =>
    state.move(0, rows),
  );
  const selected = state.selected;
  const lines = useMemo(() => {
    const canvas = creationCanvas(state.layout, state.pan, width, height);
    for (const node of state.layout.visible) {
      const p = state.layout.positions.get(node.id);
      if (
        !p ||
        p.x + nodeWidth <= state.pan.x ||
        p.x >= state.pan.x + width ||
        p.y + nodeHeight <= state.pan.y ||
        p.y >= state.pan.y + height
      )
        continue;
      const active = node.id === selected?.id;
      const card = [
        `${active ? "╔" : "┌"}${(active ? "═" : "─").repeat(nodeWidth - 2)}${active ? "╗" : "┐"}`,
        `│${fit(`${agentLabel(node.role)} ${node.id.slice(0, 6)}${state.collapsed.has(node.id) ? " +" : ""}`, nodeWidth - 2)}│`,
        `│${fit(creationStatus(node), nodeWidth - 2)}│`,
        `${active ? "╚" : "└"}${(active ? "═" : "─").repeat(nodeWidth - 2)}${active ? "╝" : "┘"}`,
      ];
      for (let row = 0; row < card.length; row++) {
        const y = p.y + row - state.pan.y;
        if (y < 0 || y >= height) continue;
        const start = Math.max(0, state.pan.x - p.x);
        const end = Math.min(nodeWidth, state.pan.x + width - p.x);
        const x = Math.max(0, p.x - state.pan.x);
        const text = card[row].slice(start, end);
        canvas[y] =
          canvas[y].slice(0, x) + text + canvas[y].slice(x + text.length);
      }
    }
    return canvas.map((content, offset) => ({
      id: `creation-row-${state.pan.y + offset}`,
      content,
    }));
  }, [state.layout, state.pan, state.collapsed, selected, width, height]);
  const creator = state.nodes.find((node) => node.id === selected?.creatorId);
  return (
    <Box flexDirection="column" minHeight={0}>
      <TerminalText wrap="truncate-end">{`Agent map · ${state.nodes.length} agents · +${state.unseen.size} new · follow ${state.follow ? "on" : "off"}`}</TerminalText>
      <TerminalText color={state.error ? "red" : undefined} wrap="truncate-end">
        {state.error
          ? `Stale/unavailable map: ${state.error}`
          : selected
            ? `${agentLabel(selected.role)} · ${creationStatus(selected)}`
            : state.loading
              ? "Reading recorded agents…"
              : "No recorded agents in this repository."}
      </TerminalText>
      <Box
        ref={target}
        width={width}
        height={height}
        flexShrink={0}
        flexDirection="column"
        overflow="hidden"
      >
        {lines.map((line) => (
          <TerminalText key={line.id} wrap="truncate-end">
            {line.content}
          </TerminalText>
        ))}
      </Box>
      <TerminalText wrap="truncate-end">
        {selected ? selected.id : "Unknown creators remain unknown."}
      </TerminalText>
      <TerminalText wrap="truncate-end">
        {selected
          ? `Creator: ${creator ? `${agentLabel(creator.role)} ${creator.id.slice(0, 6)}` : (selected.creatorId ?? "unknown")} · not Stop ownership`
          : "Creation arrows are not execution ownership."}
      </TerminalText>
      <TerminalText wrap="truncate-end">
        {selected
          ? `Gate ${selected.coordination?.gate ?? (selected.execution?.status === "needs_human" ? "needs_human" : "not projected")} · results ${selected.pendingResults} · follow-up ${selected.requiredFollowUp}`
          : "A child result is not parent/spec settlement."}
      </TerminalText>
      <TerminalText>
        Arrows pan · Tab/Shift+Tab select · Enter chat
      </TerminalText>
      <TerminalText>C collapse · Home centre · N new · F follow</TerminalText>
      <TerminalText>
        Esc back · F2 chat/map · F3 Griller · Ctrl+X Stop
      </TerminalText>
    </Box>
  );
}
