import type {
  ChildExecution,
  ChildInstruction,
  ChildResult,
  Children,
} from "../types/children.js";
import { PresentationError } from "./errors.js";

function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new PresentationError("Invalid managed-work response.");
  return value as Record<string, unknown>;
}

function text(value: unknown): string {
  if (typeof value !== "string")
    throw new PresentationError("Invalid managed-work text.");
  return value;
}

function optional(value: unknown): string | null {
  return value === null ? null : text(value);
}

function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0)
    throw new PresentationError("Invalid managed-work revision.");
  return value;
}

function flag(value: unknown): boolean {
  if (typeof value !== "boolean")
    throw new PresentationError("Invalid managed-work state.");
  return value;
}

function list(value: unknown): unknown[] {
  if (!Array.isArray(value) || value.length > 256)
    throw new PresentationError("Managed work exceeds its bounded projection.");
  return value;
}

export function parseExecution(value: unknown): ChildExecution {
  const v = object(value);
  const status = text(v.status);
  if (
    ![
      "held",
      "running",
      "completed",
      "needs_human",
      "failed",
      "stopped",
      "unknown",
    ].includes(status)
  )
    throw new PresentationError("Unknown child execution state.");
  return {
    intent: text(v.intent),
    assignmentId: text(v.assignmentId),
    executionId: text(v.executionId),
    agentId: text(v.agentId),
    creatorId: text(v.creatorId),
    parentConversationId: text(v.parentConversationId),
    parentRunId: text(v.parentRunId),
    modelId: text(v.modelId),
    providerId: text(v.providerId),
    effort: optional(v.effort),
    instructions: text(v.instructions),
    status,
    stopRequested: flag(v.stopRequested),
    outcomeId: optional(v.outcomeId),
    deliveredTo: optional(v.deliveredTo),
    followUpCompletedBy: optional(v.followUpCompletedBy),
    source: text(v.source),
  };
}

export function parseChildInstruction(value: unknown): ChildInstruction {
  const v = object(value);
  return {
    id: text(v.id),
    executionId: text(v.executionId),
    order: integer(v.order),
    text: text(v.text),
    appliedRun: optional(v.appliedRun),
    appliedReply: optional(v.appliedReply),
  };
}

export function parseChildren(value: unknown): Children | null {
  const v = object(value).coordination;
  if (v === null) return null;
  const state = object(v);
  return {
    agentId: text(state.agentId),
    conversationId: text(state.conversationId),
    revision: integer(state.revision),
    sessionRevision:
      state.sessionRevision === undefined
        ? undefined
        : integer(state.sessionRevision),
    authorityComplete: flag(state.authorityComplete),
    contextBlocked:
      state.contextBlocked === undefined ? null : flag(state.contextBlocked),

    gate: text(state.gate),
    status: text(state.status),
    settled: flag(state.settled),
    children: list(state.children).map(parseExecution),
    instructions: list(state.instructions).map(parseChildInstruction),
    joins: list(state.joins).map((item) => {
      const j = object(item);
      if (j.mode !== "any" && j.mode !== "all")
        throw new PresentationError("Invalid captured join mode.");
      return {
        id: text(j.id),
        runId: text(j.runId),
        mode: j.mode,
        executions: list(j.executions).map(text),
        backgrounded: flag(j.backgrounded),
      };
    }),
  };
}

export function parseChildResult(value: unknown): ChildResult | null {
  const result = object(value).result;
  if (result === null) return null;
  const v = object(result);
  const receipt = v.receipt === null ? null : object(v.receipt);
  if (
    receipt &&
    (text(receipt.untrustedSummary).length > 8192 ||
      list(receipt.artifacts).length > 32)
  )
    throw new PresentationError("Child result exceeds its summary bounds.");
  return {
    execution: parseExecution(v.execution),
    authority: text(v.authority),
    receipt: receipt
      ? {
          id: text(receipt.id),
          status: text(receipt.status),
          conversationId: text(receipt.conversationId),
          runId: optional(receipt.runId),
          replyId: optional(receipt.replyId),
          untrustedSummary: text(receipt.untrustedSummary),
          artifacts: list(receipt.artifacts).map(text),
        }
      : null,
  };
}
