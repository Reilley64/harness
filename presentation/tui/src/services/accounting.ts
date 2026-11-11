import { PresentationError } from "./errors.js";

export function accountingLabel(value: unknown): string {
  if (!value || typeof value !== "object")
    throw new PresentationError("Invalid usage accounting response.");
  const counter = (key: string): number | null => {
    const field: unknown = Reflect.get(value, key);
    if (field === null) return null;
    if (typeof field !== "number" || !Number.isFinite(field) || field < 0)
      throw new PresentationError("Invalid usage accounting amount.");
    return field;
  };
  const amount = (key: string) => {
    const field = counter(key);
    return field === null
      ? "unknown"
      : field === 0
        ? "0"
        : field.toPrecision(6);
  };
  return `Tokens in ${counter("inputTokens") ?? "unknown"} out ${counter("outputTokens") ?? "unknown"}; unknown ${counter("unknownTokenAttempts")} gaps ${counter("unknownHistoryGaps")}\nAPI USD ${amount("reportedUsd")}; unknown ${counter("unknownApiCostAttempts")}\nEstimate USD ${amount("estimatedUsd")} (static prices)\nSubscription attempts ${counter("subscriptionAttempts")}; native run unknown ${counter("unassignedNativeRuns")}`;
}
