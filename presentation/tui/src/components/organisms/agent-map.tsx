import type { DOMElement } from "ink";
import { Box, useWindowSize } from "ink";
import { useRef } from "react";
import { useMapPointer } from "../../hooks/use-map-pointer.js";
import type { Conversation } from "../../types/conversation.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function AgentMap({
  conversation,
  busy,
  open,
}: {
  conversation: Conversation | null;
  busy: boolean;
  open: () => void;
}) {
  const target = useRef<DOMElement>(null);
  const { columns } = useWindowSize();
  const agent = conversation?.agent;
  useMapPointer(target, agent?.id, open);
  const run = conversation?.runs?.at(-1);
  return (
    <Box flexDirection="column">
      <TerminalText>Agent map</TerminalText>
      <Box
        ref={target}
        flexDirection="column"
        width={Math.min(44, columns - 2)}
        borderStyle="round"
        borderColor="cyan"
      >
        <TerminalText bold>
          {agent
            ? agent.role === "setup"
              ? "› Setup"
              : "› Griller"
            : conversation
              ? "Unassociated pilot history"
              : "No repository Griller yet"}
        </TerminalText>
        <TerminalText>
          {busy
            ? "working · live run details pending"
            : conversation?.resumeRequired
              ? "interrupted · Resume required"
              : (run?.status ?? "ready")}
        </TerminalText>
        {agent && <TerminalText wrap="truncate-end">{agent.id}</TerminalText>}
      </Box>
      <TerminalText dimColor>
        {agent
          ? "Creator unknown · leaf execution"
          : conversation
            ? "This pilot history has no recorded agent association."
            : "Send an idea to create this repository's Griller."}
      </TerminalText>
      {conversation && (
        <>
          <TerminalText wrap="truncate-end">
            {conversation.workspace ?? "Workspace not yet bound"}
          </TerminalText>
          <TerminalText wrap="truncate-end">{`${busy ? "Last saved context · " : ""}${agent ? `${agent.role} · ` : ""}${conversation.model.providerId}/${run?.modelId ?? conversation.model.id} · ${(run ? run.effort : conversation.effort) ?? "no effort"}`}</TerminalText>
          {run && (
            <TerminalText
              dimColor
              wrap="truncate-end"
            >{`${busy ? "Last saved run" : "Run"} ${run.id}${busy ? ` · ${run.status}` : ""}`}</TerminalText>
          )}
        </>
      )}
      <TerminalText>Enter open · Esc back · Ctrl+X Stop</TerminalText>
    </Box>
  );
}
