import type { ToolCall } from "../types/tool.js";

export function toolPresentation(call: ToolCall) {
  let args: Record<string, unknown> = {};
  try {
    const parsed: unknown = JSON.parse(call.arguments);
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed))
      args = parsed as Record<string, unknown>;
  } catch {}
  const text = (key: string) =>
    typeof args[key] === "string" ? args[key] : undefined;
  const number = (key: string) =>
    typeof args[key] === "number" && Number.isFinite(args[key])
      ? args[key]
      : undefined;
  const path = text("path");
  const parameters: string[] = [];
  let input: string | undefined;
  let output = call.output;
  switch (call.name) {
    case "read": {
      if (path) parameters.push(path);
      const offset = number("offset") ?? 1;
      const limit = Math.min(2000, Math.max(1, number("limit") ?? 200));
      parameters.push(`lines ${offset}–${offset + limit - 1}`);
      break;
    }
    case "bash":
      parameters.push(`timeout ${number("timeoutSeconds") ?? 120}s`);
      input = text("command") ?? call.arguments;
      break;
    case "edit":
      if (path) parameters.push(path);
      if (Array.isArray(args.edits))
        parameters.push(
          `${args.edits.length} edit${args.edits.length === 1 ? "" : "s"}`,
        );
      break;
    case "write": {
      if (path) parameters.push(path);
      const content = text("content");
      if (content !== undefined) {
        parameters.push(`${new TextEncoder().encode(content).length} bytes`);
        if (call.status === "succeeded") output += `\n${content}`;
      }
      break;
    }
    case "ls":
      parameters.push(path ?? ".");
      break;
    case "find":
    case "grep":
    case "ast_search":
      if (text("pattern")) parameters.push(text("pattern") ?? "");
      parameters.push(path ?? ".");
      if (text("language")) parameters.push(text("language") ?? "");
      if (text("glob")) parameters.push(`glob ${text("glob")}`);
      if (args.literal === true) parameters.push("literal");
      if (args.ignoreCase === true) parameters.push("ignore case");
      if (number("contextLines") !== undefined)
        parameters.push(`context ${number("contextLines")}`);
      break;
    case "web_search":
      if (text("query")) parameters.push(text("query") ?? "");
      break;
    default:
      input = call.arguments;
  }
  if (!["read", "bash"].includes(call.name) && number("limit") !== undefined)
    parameters.push(`limit ${number("limit")}`);
  if (call.status !== "succeeded")
    parameters.unshift(
      call.status === "unknown"
        ? "Unknown · effects must be inspected"
        : call.status === "requested"
          ? "Pending"
          : call.status.charAt(0).toUpperCase() + call.status.slice(1),
    );
  return {
    title: [call.name, ...parameters].join(" · ").slice(0, 1000),
    input,
    output,
    path,
  };
}
