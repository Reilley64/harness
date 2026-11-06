import type { ToolCall } from "../types/tool.js";
import { toolStatuses } from "../types/tool.js";
import { PresentationError } from "./errors.js";

export function parseTool(value: unknown): ToolCall {
  if (typeof value !== "object" || value === null)
    throw new PresentationError("Invalid tool update.");
  const v = value as Record<string, unknown>;
  const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
  const status = toolStatuses.find((status) => status === v.status);
  if (
    typeof v.id !== "string" ||
    !uuid.test(v.id) ||
    typeof v.messageId !== "string" ||
    !uuid.test(v.messageId) ||
    typeof v.name !== "string" ||
    v.name.length > 256 ||
    typeof v.arguments !== "string" ||
    new TextEncoder().encode(v.arguments).length > 65536 ||
    !status ||
    typeof v.output !== "string" ||
    new TextEncoder().encode(v.output).length > 65536 ||
    (v.artifact !== null && typeof v.artifact !== "string") ||
    typeof v.truncated !== "boolean" ||
    !Number.isSafeInteger(v.elapsedMs) ||
    Number(v.elapsedMs) < 0 ||
    !Number.isSafeInteger(v.textOffset) ||
    Number(v.textOffset) < 0
  )
    throw new PresentationError("Invalid tool update.");
  return {
    id: v.id,
    messageId: v.messageId,
    name: v.name,
    arguments: v.arguments,
    status,
    output: v.output,
    artifact: v.artifact,
    truncated: v.truncated,
    elapsedMs: Number(v.elapsedMs),
    textOffset: Number(v.textOffset),
  };
}
