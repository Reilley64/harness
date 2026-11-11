import type { WorkResponse } from "../types/work.js";
import { PresentationError } from "./errors.js";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
export function parseWork(value: unknown): WorkResponse {
  if (
    !record(value) ||
    !(
      value.conversationId === null || typeof value.conversationId === "string"
    ) ||
    !(value.work === null || record(value.work))
  )
    throw new PresentationError("Invalid approved-work response.");
  if (value.work !== null) {
    const w = value.work;
    if (
      typeof w.id !== "string" ||
      typeof w.creatorId !== "string" ||
      typeof w.sessionId !== "string" ||
      typeof w.workspace !== "string" ||
      !Number.isSafeInteger(w.revision) ||
      !Array.isArray(w.publications) ||
      !w.publications.length ||
      !Array.isArray(w.approvals) ||
      !Array.isArray(w.unavailable) ||
      w.complete !== false
    )
      throw new PresentationError("Invalid approved work.");
    for (const p of w.publications)
      if (
        !record(p) ||
        typeof p.intent !== "string" ||
        typeof p.version !== "string" ||
        typeof p.document !== "string"
      )
        throw new PresentationError("Invalid published proposal.");
    for (const a of w.approvals)
      if (
        !record(a) ||
        typeof a.intent !== "string" ||
        typeof a.version !== "string" ||
        (a.transientRetryAllowance != null &&
          (typeof a.transientRetryAllowance !== "number" ||
            !Number.isSafeInteger(a.transientRetryAllowance) ||
            a.transientRetryAllowance < 0 ||
            a.transientRetryAllowance > 64)) ||
        typeof a.conversationId !== "string" ||
        typeof a.delivered !== "boolean" ||
        typeof a.current !== "boolean"
      )
        throw new PresentationError("Invalid work approval.");
  }
  return value as WorkResponse;
}
