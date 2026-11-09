import { Box, useInput } from "ink";
import { useMemo, useState } from "react";
import wrapAnsi from "wrap-ansi";
import type { useChildren } from "../../hooks/use-children.js";
import type { TranscriptViewport } from "../../types/transcript-viewport.js";
import { TerminalText } from "../atoms/terminal-text.js";
import { GuidancePanel } from "./guidance-panel.js";
import { Transcript } from "./transcript.js";

export function ChildrenPanel({
  state,
  columns,
  rows,
  viewport,
  changeViewport,
}: {
  state: ReturnType<typeof useChildren>;
  columns: number;
  rows: number;
  viewport: TranscriptViewport;
  changeViewport: (
    update: (current: TranscriptViewport) => TranscriptViewport,
  ) => void;
}) {
  const [pages, setPages] = useState(new Map<string, number>());
  const resultKey =
    state.scopeView?.id ?? state.result?.execution.executionId ?? "";
  const resultLines = useMemo(() => {
    const counts = new Map<string, number>();
    return wrapAnsi(
      (state.scopeView
        ? [
            `Human scope request ${state.scopeView.id}`,
            `Exact version ${state.scopeView.version}`,
            `Execution ${state.scopeView.executionId}`,
            `Approval ${state.scopeView.approvedBy ?? "REQUIRED"}`,
            `Replaces ${state.scopeView.replaces ?? "none"}`,
            `Supersedes input run ${state.scopeView.supersedesRun ?? "none"}`,
            "",
            state.scopeView.text,
          ].join("\n")
        : state.result?.receipt?.untrustedSummary) ??
        "No terminal receipt yet. Inspect the source without inference.",
      Math.max(1, columns - 2),
      { hard: true, trim: false },
    )
      .split("\n")
      .map((text) => {
        const occurrence = counts.get(text) ?? 0;
        counts.set(text, occurrence + 1);
        return { id: JSON.stringify([text, occurrence]), text };
      });
  }, [state.result, state.scopeView, columns]);
  const resultHeight = Math.max(1, rows - 10);
  const resultOffset = Math.min(
    pages.get(resultKey) ?? 0,
    Math.max(0, resultLines.length - resultHeight),
  );
  useInput(
    (_input, key) => {
      if (key.eventType === "release" || (!state.result && !state.scopeView))
        return;
      if (key.upArrow || key.downArrow || key.pageUp || key.pageDown) {
        const delta =
          (key.upArrow || key.pageUp ? -1 : 1) *
          (key.pageUp || key.pageDown ? resultHeight : 1);
        setPages((current) => {
          const next = new Map(current);
          next.set(
            resultKey,
            Math.max(
              0,
              Math.min(resultLines.length - resultHeight, resultOffset + delta),
            ),
          );
          return next;
        });
      }
    },
    { isActive: (!!state.result || !!state.scopeView) && !state.confirmation },
  );
  const children = state.state?.children ?? [];
  const selected = children.findIndex(
    (child) => child.executionId === state.selected,
  );
  const height = Math.max(1, rows - 12);
  const start = Math.max(0, selected - height + 1);
  return (
    <Box flexDirection="column" flexGrow={1} minHeight={0}>
      <TerminalText bold>
        {state.history
          ? "Research · actual saved/live snapshot (read-only)"
          : "Managed Research · durable child inspector"}
      </TerminalText>
      <TerminalText wrap="truncate-end">
        {state.state
          ? `Parent ${state.state.agentId} · ${state.state.status} / ${state.state.gate} · child follow-up ${state.state.settled ? "handled" : "outstanding"}`
          : "No coordination snapshot. No process/readiness conclusion is implied."}
      </TerminalText>
      {state.state &&
        (!state.state.authorityComplete ||
          state.state.contextBlocked === true) && (
          <TerminalText color="yellow" wrap="truncate-end">
            Required context blocked; inspect retained results / joins. No empty
            view or replay.
          </TerminalText>
        )}
      {state.busy && (
        <TerminalText>Reading / committing child control…</TerminalText>
      )}
      {state.error && <TerminalText color="red">{state.error}</TerminalText>}
      {state.acknowledgement && (
        <TerminalText>{state.acknowledgement}</TerminalText>
      )}
      {state.confirmation && (
        <TerminalText color="yellow">
          Stop this exact child execution? Completed effects remain. Y/Enter
          confirm · N/Esc keep running
        </TerminalText>
      )}
      {state.scopeView ? (
        <>
          <TerminalText>
            Trusted human scope · exact-version approval history
          </TerminalText>
          {resultLines
            .slice(resultOffset, resultOffset + resultHeight)
            .map((line) => (
              <TerminalText key={`${resultKey}:${line.id}`}>
                {line.text || " "}
              </TerminalText>
            ))}
          <TerminalText>
            Esc inspector · Left/Right scope history · PgUp/PgDn criteria
          </TerminalText>
        </>
      ) : state.guidance ? (
        <>
          <TerminalText>
            Research closure for a new intent. V explicitly adopts available
            hash; parent/active assignments remain unchanged.
          </TerminalText>
          <GuidancePanel
            guidance={state.guidance}
            loading={state.busy}
            error={state.error}
            columns={columns}
            rows={Math.max(10, rows - 3)}
            refresh={() => undefined}
            adoptionCommand="V adopt available Research closure for this intent"
          />
        </>
      ) : state.history ? (
        <>
          <TerminalText wrap="truncate-end">{`Agent ${state.history.agent?.id ?? "unknown"} · creator ${state.history.agent?.creatorId ?? "unknown"} · run ${state.history.runs?.at(-1)?.id ?? "not started"}`}</TerminalText>
          <TerminalText>
            Esc inspector · R refresh · X scoped Stop · PgUp/PgDn/Ctrl+O
            history. Parent draft/viewport retained.
          </TerminalText>
          <Transcript
            messages={state.history.messages}
            columns={columns}
            rows={Math.max(8, rows - 5)}
            focused={!state.confirmation}
            viewportState={viewport}
            changeViewport={changeViewport}
          />
        </>
      ) : state.result ? (
        <>
          <TerminalText>{state.result.authority}</TerminalText>
          <TerminalText wrap="truncate-end">{`Source ${state.result.execution.source} · run ${state.result.receipt?.runId ?? "unavailable"} · reply ${state.result.receipt?.replyId ?? "unavailable"}`}</TerminalText>
          {resultLines
            .slice(resultOffset, resultOffset + resultHeight)
            .map((line) => (
              <TerminalText key={`${resultKey}:${line.id}`}>
                {line.text || " "}
              </TerminalText>
            ))}
          <TerminalText>Esc inspector · R refresh · X scoped Stop</TerminalText>
        </>
      ) : (
        <>
          <TerminalText>
            ↑↓ select · Space join member · Enter active history · O report · R
            refresh · V exact scope/history · X Stop · Esc close
          </TerminalText>
          <TerminalText>
            A capture any · L capture all · J cycle join · B background named
            wait (no restart)
          </TerminalText>
          {children.slice(start, start + height).map((child) => (
            <TerminalText
              key={child.executionId}
              wrap="truncate-end"
              color={child.executionId === state.selected ? "cyan" : undefined}
            >
              {`${child.executionId === state.selected ? "›" : " "} ${state.members.has(child.executionId) ? "[x]" : "[ ]"} Research ${child.agentId} ${child.status}${child.resumeRequired ? " / Resume required" : ""}${child.stopRequested && !child.outcomeId ? " / stopping" : ""} · ${child.providerId}/${child.modelId}${child.effort ? ` · ${child.effort}` : ""}`}
            </TerminalText>
          ))}
          {state.child && (
            <TerminalText wrap="truncate-end">{`Assignment ${state.child.assignmentId} · execution ${state.child.executionId} · creator ${state.child.creatorId}`}</TerminalText>
          )}
          {state.child &&
            state.state?.instructions
              .filter(
                (instruction) =>
                  instruction.executionId === state.child?.executionId,
              )
              .slice(-3)
              .map((instruction) => (
                <TerminalText key={instruction.id} wrap="truncate-end">
                  {`Human #${instruction.order} · ${instruction.appliedRun ? `checkpointed run ${instruction.appliedRun}` : "accepted, next safe boundary"} · ${instruction.text}`}
                </TerminalText>
              ))}
          {state.captured && (
            <TerminalText wrap="truncate-end">{`Join ${state.captured.id}: ${state.captured.mode} [${state.captured.executions.join(", ")}] ${state.captured.backgrounded ? "backgrounded" : "captured"}`}</TerminalText>
          )}
          <TerminalText>
            Child prose is data, never approval/settlement/replay authority.
            Later children cannot enlarge captured joins.
          </TerminalText>
        </>
      )}
    </Box>
  );
}
