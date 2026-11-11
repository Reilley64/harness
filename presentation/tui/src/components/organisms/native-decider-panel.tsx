import { Box, useInput } from "ink";
import { useEffect, useMemo, useState } from "react";
import wrapAnsi from "wrap-ansi";
import type { Deciders, DecisionResponse } from "../../types/decision.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function NativeDeciderPanel({
  mode,
  settings,
  view,
  loading,
  saving,
  pending,
  error,
  columns,
  rows,
  choose,
  select,
  stop,
  configure,
  discuss,
}: {
  mode: "picker" | "inspector";
  settings: Deciders | null;
  view: DecisionResponse | null;
  loading: boolean;
  saving: boolean;
  pending: boolean;
  error: string | null;
  columns: number;
  rows: number;
  choose: (id: string) => void;
  select: () => void;
  stop: () => void;
  configure: () => void;
  discuss: (id: string) => void;
}) {
  const [index, setIndex] = useState(0),
    [offset, setOffset] = useState(0),
    [confirm, setConfirm] = useState(false);
  const height = Math.max(1, rows - 9);
  const entries = settings?.models ?? [];
  const selected = entries[Math.min(index, Math.max(0, entries.length - 1))];
  const lines = useMemo(() => {
    const text =
      mode === "picker"
        ? [
            "Separate native Decisions releases; no native thinking control.",
            "Moving aliases may resolve to a different release. Hosted routers disabled.",
            "Saved selection: " + (settings?.selected ?? "none"),
            ...entries.map(
              (m, i) =>
                (i === index ? "> " : "  ") +
                m.id +
                (m.id.startsWith("~")
                  ? " · MOVING ALIAS"
                  : " · explicit release") +
                (m.blocked ? " · " + m.blocked : ""),
            ),
            "Connecting does not prove entitlement. Saving never infers or broadens policy.",
          ]
        : [
            "No worker launched · typed selection/inspection only",
            "Free-form discussion is with Chief of Staff, not native Decider (o).",
            "phase: " +
              (pending ? "pending" : (view?.decision?.phase ?? "unavailable")),
            pending
              ? "Pending without an application/SDK deadline. Esc navigates; x then y explicitly Stops."
              : "",
            ...(view?.blocked
              ? [
                  "CONFIGURATION REQUIRED · no replacement: " + view.blocked,
                  "c opens existing provider/model settings; Escape preserves blocked work.",
                ]
              : []),
            view?.decision?.details ?? "Assignment intent unavailable.",
            "Reported usage is distinct from list prices; missing cost/benchmarks/confidence remain unknown, not zero.",
          ];
    const counts = new Map<string, number>();
    return text
      .concat(
        error ?? "",
        loading ? "Loading…" : saving ? "Saving durable preference…" : "",
      )
      .flatMap((t) => t.split("\n"))
      .flatMap((t) =>
        wrapAnsi(t.replace(/\p{Cc}/gu, ""), Math.max(1, columns - 2), {
          hard: true,
          trim: false,
        }).split("\n"),
      )
      .map((text) => {
        const count = counts.get(text) ?? 0;
        counts.set(text, count + 1);
        return { id: `${text}:${count}`, text };
      });
  }, [
    mode,
    settings,
    entries,
    index,
    view,
    pending,
    error,
    loading,
    saving,
    columns,
  ]);
  const selectionLine = lines.findIndex((line) => line.text.startsWith("> "));
  useEffect(() => {
    if (mode !== "picker" || selectionLine < 0) return;
    setOffset((current) =>
      selectionLine < current
        ? selectionLine
        : selectionLine >= current + height
          ? selectionLine - height + 1
          : current,
    );
  }, [mode, selectionLine, height]);
  const start = Math.min(offset, Math.max(0, lines.length - height));
  useInput((input, key) => {
    if (key.eventType === "release") return;
    if (mode === "picker") {
      if (key.upArrow || key.downArrow)
        setIndex(
          Math.max(
            0,
            Math.min(entries.length - 1, index + (key.upArrow ? -1 : 1)),
          ),
        );
      else if (
        key.return &&
        selected &&
        !loading &&
        !saving &&
        !selected.blocked
      )
        choose(selected.id);
    } else {
      if (input === "s" && !pending) select();
      else if (input === "x" || (key.ctrl && input === "x")) setConfirm(true);
      else if (confirm && input === "y") {
        setConfirm(false);
        stop();
      } else if (input === "n" || key.escape) setConfirm(false);
      else if (input === "c") configure();
      else if (input === "o" && view?.decision) discuss(view.decision.parentId);
    }
    if (
      key.pageUp ||
      key.pageDown ||
      (mode === "inspector" && (key.upArrow || key.downArrow))
    ) {
      const step = key.pageUp || key.pageDown ? height : 1;
      setOffset(
        Math.max(
          0,
          Math.min(
            lines.length - height,
            start + (key.upArrow || key.pageUp ? -step : step),
          ),
        ),
      );
    }
  });
  return (
    <Box flexDirection="column">
      <TerminalText bold>
        {mode === "picker"
          ? "Native Decider releases"
          : "Native decision inspector · No worker launched"}
      </TerminalText>
      {lines.slice(start, start + height).map((line) => (
        <TerminalText key={line.id} wrap="truncate-end">
          {line.text || " "}
        </TerminalText>
      ))}
      <TerminalText>
        {confirm
          ? "y STOP native request · n back · no fallback"
          : mode === "picker"
            ? "↑↓ select · Enter save · Esc close · PgUp/PgDn inspect"
            : "s select once · x Stop · c configure · o CoS · Esc back"}
      </TerminalText>
    </Box>
  );
}
