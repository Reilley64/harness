import { Box, useInput } from "ink";
import { useMemo, useState } from "react";
import wrapAnsi from "wrap-ansi";
import type { Setup } from "../../types/setup.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function SetupPanel({
  view,
  loading,
  saving,
  error,
  columns,
  rows,
  refresh,
  update,
}: {
  view: Setup;
  loading: boolean;
  saving: boolean;
  error: string | null;
  columns: number;
  rows: number;
  refresh: () => void;
  update: (id: string, confirm: boolean) => void;
}) {
  const [index, setIndex] = useState(0);
  const [offset, setOffset] = useState(0);
  const [confirmation, setConfirmation] = useState<string | null>(null);
  const proposal = view.proposals[Math.min(index, view.proposals.length - 1)];
  const lines = useMemo(() => {
    const counts = new Map<string, number>();
    return [
      ...(loading ? ["Inspecting repository…"] : []),
      ...(error ? [error] : []),
      view.missing.length
        ? `Missing: ${view.missing.join(", ")}`
        : "All applicable output files are present. Setup is bypassed on repository open.",
      `Roots: ${view.roots.join(", ") || "none — agree which to create"} · triage ${view.triage ? "applicable" : "not selected"}`,
      "Setup fills missing guidance only. Chat to agree tracker/domain/wiki choices. Existing wiki history is not replaced.",
      "Bash remains an OS-user trusted tool, not a sandbox. External/machine-wide work requires separate consent and is not performed by this confirmation.",
      ...(proposal
        ? [
            `Proposal ${index + 1}/${view.proposals.length} · ${proposal.status} · ${proposal.id}`,
            `Target: ${exact(proposal.draft.path)} · revision ${view.revision}`,
            "BEFORE (preserved-content base):",
            proposal.draft.before === null
              ? "(absent)"
              : exact(proposal.draft.before),
            "AFTER (JSON-escaped exact proposed text; escapes are displayed, not written):",
            exact(proposal.draft.after),
            `Result: ${proposal.output}`,
          ]
        : [
            "No stored drafts. Continue the Setup conversation to inspect and propose the missing outputs.",
          ]),
    ]
      .flatMap((line) => line.split("\n"))
      .flatMap((line) =>
        wrapAnsi(line.replace(/\p{Cc}/gu, ""), Math.max(1, columns - 2), {
          hard: true,
          trim: false,
        }).split("\n"),
      )
      .map((text) => {
        const count = counts.get(text) ?? 0;
        counts.set(text, count + 1);
        return { id: `${text}:${count}`, text };
      });
  }, [view, loading, error, proposal, index, columns]);
  const height = Math.max(1, rows - 6);
  const start = Math.min(offset, Math.max(0, lines.length - height));
  useInput((input, key) => {
    if (key.eventType === "release" || key.eventType === "repeat") return;
    if (confirmation) {
      if (key.escape || input === "n") setConfirmation(null);
      else if (input === "y" && !saving && confirmation === proposal?.id) {
        setConfirmation(null);
        update(confirmation, true);
      }
      return;
    }
    if (saving || loading) return;
    if (input === "r") refresh();
    else if (key.leftArrow || key.rightArrow) {
      setIndex(
        Math.max(
          0,
          Math.min(view.proposals.length - 1, index + (key.leftArrow ? -1 : 1)),
        ),
      );
      setOffset(0);
    } else if (key.pageUp || key.pageDown || key.upArrow || key.downArrow)
      setOffset(
        Math.max(
          0,
          Math.min(
            lines.length - height,
            start +
              (key.pageUp || key.upArrow ? -1 : 1) *
                (key.pageUp || key.pageDown ? height : 1),
          ),
        ),
      );
    else if (proposal?.status === "draft" && input === "c")
      setConfirmation(proposal.id);
    else if (proposal?.status === "draft" && input === "x")
      update(proposal.id, false);
  });
  return (
    <Box flexDirection="column">
      <TerminalText>Repository Setup · Esc closes, never Stop</TerminalText>
      <TerminalText>
        {confirmation
          ? `Approve ONLY ${exact(proposal?.draft.path ?? "")} at shown revision? y confirms · n/Esc cancels`
          : saving
            ? "Recording confirmed effect; closing does not undo it…"
            : "←→ proposal · ↑↓/PgUp/PgDn inspect · c confirm · x cancel draft · r refresh"}
      </TerminalText>
      {lines.slice(start, start + height).map((line) => (
        <TerminalText key={line.id} wrap="truncate-end">
          {line.text || " "}
        </TerminalText>
      ))}
    </Box>
  );
}

function exact(value: string): string {
  return JSON.stringify(value).replace(
    /[^\u0020-\u007e]/g,
    (character) =>
      `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`,
  );
}
