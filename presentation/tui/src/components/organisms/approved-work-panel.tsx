import { Box, useInput } from "ink";
import { useMemo, useState } from "react";
import wrapAnsi from "wrap-ansi";
import type { Conversation } from "../../types/conversation.js";
import type { ApprovedWork } from "../../types/work.js";
import { TerminalText } from "../atoms/terminal-text.js";
export function ApprovedWorkPanel({
  work,
  conversation,
  loading,
  saving,
  error,
  coordinator,
  columns,
  rows,
  approve,
  openCoordinator,
}: {
  work: ApprovedWork | null;
  conversation: Conversation | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  coordinator: string | null;
  columns: number;
  rows: number;
  approve: (transientRetryAllowance?: number) => void;
  openCoordinator: () => void;
}) {
  const [offset, setOffset] = useState(0),
    [confirm, setConfirm] = useState(false),
    [retryDraft, setRetryDraft] = useState<number | undefined>(undefined);
  const currentApproval = work?.approvals.find((approval) => approval.current);
  const retryAllowance = currentApproval
    ? (currentApproval.transientRetryAllowance ?? undefined)
    : retryDraft;
  const retryLabel = `${retryAllowance ?? 2} (${retryAllowance === undefined ? "default" : "explicit"})`;

  const lines = useMemo(() => {
    const publication = work?.publications.at(-1);
    const content = [
      "Approved work · exact source document (not mutable tracker readiness)",
      ...(work && publication
        ? [
            "Work: " + work.id + " · revision " + work.revision,
            "Scope/criteria VERSION: " + publication.version,
            "Source/spec/scope/criteria/dependencies/wiki/context/artifacts:",
            publication.document,
            "Griller selection to retain: " +
              (conversation
                ? conversation.model.providerId +
                  "/" +
                  conversation.model.id +
                  " · " +
                  (conversation.effort ?? "no effort")
                : "unavailable"),
            "Initial automatic transient retries: " +
              retryLabel +
              (currentApproval
                ? " · recorded approval (immutable)"
                : " · -/+ adjust 0..64 · d default"),
            "Approval authorizes ONLY this version's implementation scope; no live auth, provisioning or merge consent is implied. Managed child execution is unavailable.",
            ...work.approvals.map(
              (a) =>
                "Intent " +
                a.intent +
                " · " +
                (a.current ? "matching version" : "STALE version") +
                " · " +
                (a.delivered ? "delivered" : "handoff pending") +
                " · CoS " +
                a.conversationId,
            ),
            "Unavailable: " + work.unavailable.join(", "),
          ]
        : []),
      ...(coordinator
        ? [
            "Delivered to separate durable Chief of Staff: " +
              coordinator +
              " · o opens (no inference)",
          ]
        : []),
      ...(error ? [error] : []),
      loading
        ? "Loading…"
        : saving
          ? "Saving durable approval/handoff… Escape may dismiss but does not undo an admitted save."
          : confirm
            ? `Confirm exact version/scope and initial transient retries ${retryLabel}? y APPROVE · n back`
            : "a review approval confirmation · Esc close · ↑↓/PgUp/PgDn inspect",
    ];
    const counts = new Map<string, number>();
    return content
      .flatMap((v) => v.split("\n"))
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
  }, [
    work,
    conversation,
    loading,
    saving,
    error,
    coordinator,
    columns,
    confirm,
    currentApproval,
    retryLabel,
  ]);
  const height = Math.max(1, rows - 6),
    start = Math.min(offset, Math.max(0, lines.length - height));
  useInput((input, key) => {
    if (key.eventType === "release") return;
    if (confirm && input === "n") setConfirm(false);
    else if (confirm && input === "y" && !saving && !loading) {
      setConfirm(false);
      approve(retryAllowance);
    } else if (
      !confirm &&
      !saving &&
      !loading &&
      !currentApproval &&
      work &&
      (input === "+" || input === "-" || input === "d")
    ) {
      setRetryDraft(
        input === "d"
          ? undefined
          : Math.max(
              0,
              Math.min(64, (retryAllowance ?? 2) + (input === "+" ? 1 : -1)),
            ),
      );
    } else if (input === "a" && work && !saving && !loading) setConfirm(true);
    else if (input === "o" && coordinator && !saving) openCoordinator();
    else if (key.upArrow || key.downArrow || key.pageUp || key.pageDown) {
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
        {`Approved work · a confirm · Esc dismiss · ${start + 1}/${lines.length}`}
      </TerminalText>
      {lines.slice(start, start + height).map((line) => (
        <TerminalText key={line.id} wrap="truncate-end">
          {line.text || " "}
        </TerminalText>
      ))}
      <TerminalText>
        {confirm
          ? `y APPROVE exact version · retries ${retryLabel} · n back`
          : saving
            ? "Saving…"
            : `retries ${retryLabel} · -/+ adjust · d default · a approve · o CoS`}
      </TerminalText>
    </Box>
  );
}
