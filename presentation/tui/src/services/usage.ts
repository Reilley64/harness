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
  const wait: unknown = Reflect.get(value, "providerWait");
  let providerWait: Usage["providerWait"] = null;
  if (wait != null) {
    if (
      typeof wait !== "object" ||
      !("kind" in wait) ||
      (wait.kind !== "transient" && wait.kind !== "quota") ||
      !("failures" in wait) ||
      typeof wait.failures !== "number" ||
      !Number.isSafeInteger(wait.failures) ||
      wait.failures < 1 ||
      !("allowance" in wait) ||
      typeof wait.allowance !== "number" ||
      !Number.isSafeInteger(wait.allowance) ||
      wait.allowance < 2 ||
      !("retryAt" in wait) ||
      (wait.retryAt !== null &&
        (typeof wait.retryAt !== "number" ||
          !Number.isSafeInteger(wait.retryAt) ||
          wait.retryAt < 0))
    )
      throw new PresentationError("Invalid provider wait.");
    providerWait = {
      kind: wait.kind,
      failures: wait.failures,
      allowance: wait.allowance,
      retryAt: wait.retryAt,
    };
  }
  return {
    providerWait,
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
  const wait = usage?.providerWait;
  if (wait) {
    if (wait.retryAt === 0) return "Retry ready · /resume if recovery-gated";
    if (wait.retryAt === null)
      return wait.kind === "quota"
        ? "Quota wait · /resume; reset unknown"
        : "Retries exhausted · /retry extend <n>";
    return `${wait.kind === "quota" ? "Quota wait" : "Retry wait"} · ${new Date(wait.retryAt).toISOString().slice(11, 19)} UTC · Stop available`;
  }
  return `in ${count(usage?.inputTokens)} out ${count(usage?.outputTokens)} · ctx ${window}`;
}
