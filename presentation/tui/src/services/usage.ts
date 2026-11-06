import type { Model } from "../types/model.js";
import type { Usage } from "../types/usage.js";
import { PresentationError } from "./errors.js";

export function parseUsage(value: unknown): Usage {
  if (!value || typeof value !== "object")
    throw new PresentationError("Invalid token usage.");
  const field = (name: string): number | null => {
    const counter = Reflect.get(value, name);
    if (counter === null) return null;
    if (
      typeof counter !== "number" ||
      !Number.isSafeInteger(counter) ||
      counter < 0
    )
      throw new PresentationError("Invalid token usage counter.");
    return counter;
  };
  const modelId: unknown = Reflect.get(value, "modelId");
  const partial: unknown = Reflect.get(value, "partial");
  if (
    (modelId !== null && typeof modelId !== "string") ||
    typeof partial !== "boolean"
  )
    throw new PresentationError("Invalid token usage metadata.");
  return {
    inputTokens: field("inputTokens"),
    outputTokens: field("outputTokens"),
    contextTokens: field("contextTokens"),
    modelId,
    partial,
  };
}

export function parseUsageUpdate(value: unknown) {
  if (
    !value ||
    typeof value !== "object" ||
    !("sessionId" in value) ||
    typeof value.sessionId !== "string" ||
    !("usage" in value)
  )
    throw new PresentationError("Invalid usage update.");
  return { sessionId: value.sessionId, usage: parseUsage(value.usage) };
}

export function usageLabel(
  usage: Usage | null | undefined,
  model: Model | undefined,
): string {
  const compact = (value: number | null | undefined) =>
    value == null
      ? "—"
      : new Intl.NumberFormat("en", {
          notation: "compact",
          maximumFractionDigits: 1,
        })
          .format(value)
          .toLowerCase();
  const count = (value: number | null | undefined) =>
    `${usage?.partial && value != null ? "≥" : ""}${compact(value)}`;
  const limit = model?.contextWindow;
  const context = usage?.modelId === model?.id ? usage?.contextTokens : null;
  const window =
    limit && context != null ? `${((context / limit) * 100).toFixed(1)}%` : "—";
  return `in ${count(usage?.inputTokens)} out ${count(usage?.outputTokens)} · ctx ${window}`;
}
