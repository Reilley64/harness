import type { Deciders, DecisionResponse } from "../types/decision.js";
import { PresentationError } from "./errors.js";
import { parseModels } from "./models.js";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function parseDecision(value: unknown): DecisionResponse {
  if (
    !record(value) ||
    !(value.blocked === null || typeof value.blocked === "string")
  )
    throw new PresentationError("Invalid decision response.");
  if (value.decision === null)
    return { decision: null, blocked: value.blocked };
  const d = value.decision;
  if (
    !record(d) ||
    typeof d.id !== "string" ||
    typeof d.parentId !== "string" ||
    typeof d.revision !== "number" ||
    !Number.isSafeInteger(d.revision) ||
    d.revision < 1 ||
    typeof d.phase !== "string" ||
    ![
      "prepared",
      "pending",
      "selected",
      "fallback",
      "configuration",
      "unsupported",
      "stopped",
    ].includes(d.phase) ||
    d.workerLaunched !== false
  )
    throw new PresentationError("Invalid assignment decision.");
  if (
    !record(d.brief) ||
    (d.brief.role !== "research" &&
      d.brief.role !== "implementer" &&
      d.brief.role !== "reviewer")
  )
    throw new PresentationError("Invalid assignment role.");
  return {
    decision: {
      id: d.id,
      role: d.brief.role,
      parentId: d.parentId,
      revision: d.revision,
      phase: d.phase,
      details: JSON.stringify(d, null, 2),
    },
    blocked: value.blocked,
  };
}

export function parseDeciders(value: unknown): Deciders {
  if (
    !record(value) ||
    !(value.selected === null || typeof value.selected === "string")
  )
    throw new PresentationError("Invalid native Decider settings.");
  return { models: parseModels(value), selected: value.selected };
}
