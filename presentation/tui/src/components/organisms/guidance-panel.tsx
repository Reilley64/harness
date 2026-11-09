import { Box, useInput } from "ink";
import { useMemo, useState } from "react";
import wrapAnsi from "wrap-ansi";
import type { Guidance } from "../../types/guidance.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function GuidancePanel({
  guidance,
  loading,
  error,
  columns,
  rows,
  refresh,
  adoptionCommand = "/guidance adopt",
}: {
  guidance: Guidance | null;
  loading: boolean;
  error: string | null;
  columns: number;
  rows: number;
  refresh: () => void;
  adoptionCommand?: string;
}) {
  const [tab, setTab] = useState<
    "selected" | "current" | "available" | "bundled"
  >("selected");
  const [offset, setOffset] = useState(0);
  const [body, setBody] = useState(false);
  const lines = useMemo(() => {
    if (!guidance)
      return [
        { id: "status", text: error ?? "Resolving required instructions…" },
      ];
    const snapshot = tab === "bundled" ? null : guidance[tab];
    const instructions =
      tab === "bundled" ? guidance.bundled : (snapshot?.instructions ?? []);
    const content = [
      `View: ${tab} · ${tab === "bundled" ? "Assets only — later execution/dispatch unavailable" : snapshot ? `Phase: ${snapshot.phase}` : "Unknown / not captured"}`,
      `Selected: ${guidance.selected?.id ?? "none"}`,
      `Current run: ${guidance.currentRun ?? "none"} · snapshot ${guidance.current?.id ?? "unknown"}`,
      `Available: ${guidance.available?.id ?? "BLOCKED"}`,
      guidance.selected &&
      guidance.available &&
      guidance.selected.id !== guidance.available.id
        ? "Update available; ongoing work retains captured text. Adoption is explicit."
        : "No silent updates; Resume uses the captured run snapshot.",
      ...guidance.blocked.map((value) => `REQUIRED BLOCK: ${value}`),
      ...guidance.overrides.map((value) => `Explicit override: ${value}`),
      ...(guidance.historicalUnknown.length
        ? [
            `Historical provenance unknown for ${guidance.historicalUnknown.length} run(s); no reconstruction.`,
          ]
        : []),
      ...(snapshot
        ? [
            `Scopes: ${snapshot.scopes.join(", ") || "root only"}`,
            `References: ${snapshot.references.join(", ") || "none"}`,
          ]
        : []),
      ...instructions.flatMap((instruction) => [
        "",
        instruction.id,
        `Version: ${instruction.version}`,
        `Source: ${instruction.source}`,
        ...instruction.upstream.map((source) => `Upstream: ${source}`),
        ...(!instruction.upstream.length
          ? [
              "Upstream: Harness contract or explicit content; no upstream identity asserted",
            ]
          : []),
        `Requires: ${instruction.requires.join(", ") || "none"}`,
        ...(body ? instruction.body.split("\n") : []),
      ]),
    ];
    const counts = new Map<string, number>();
    return content
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
  }, [guidance, error, tab, body, columns]);
  const height = Math.max(1, rows - 5);
  const start = Math.min(offset, Math.max(0, lines.length - height));
  useInput((input, key) => {
    if (key.eventType === "release") return;
    const next = {
      s: "selected",
      c: "current",
      a: "available",
      b: "bundled",
    } as const;
    if (input in next) {
      setTab(next[input as keyof typeof next]);
      setOffset(0);
    } else if (input === "d") {
      setBody((value) => !value);
      setOffset(0);
    } else if (input === "r" && !loading) refresh();
    else if (key.upArrow || key.downArrow || key.pageUp || key.pageDown) {
      const direction = key.upArrow || key.pageUp ? -1 : 1;
      setOffset(
        Math.max(
          0,
          Math.min(
            lines.length - height,
            start + direction * (key.pageUp || key.pageDown ? height : 1),
          ),
        ),
      );
    }
  });
  return (
    <Box flexDirection="column">
      <TerminalText>
        Instruction provenance · Esc closes (never Stop)
      </TerminalText>
      <TerminalText>
        s selected · c run · a available · b bundled · d text · r refresh
      </TerminalText>
      <TerminalText>{`↑↓/PgUp/PgDn · ${start + 1}/${lines.length} · ${adoptionCommand}`}</TerminalText>
      {lines.slice(start, start + height).map((line) => (
        <TerminalText key={line.id} wrap="truncate-end">
          {line.text || " "}
        </TerminalText>
      ))}
    </Box>
  );
}
