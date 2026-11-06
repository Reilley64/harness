import wrapAnsi from "wrap-ansi";
import type { Message, MessageStatus } from "../types/conversation.js";
import type { ToolCall } from "../types/tool.js";
import type { TranscriptLine, TranscriptTone } from "../types/transcript.js";
import { highlightCode } from "./highlight-code.js";
import { toolPresentation } from "./tool-presentation.js";

const statusLabels: Record<
  Exclude<MessageStatus, "completed" | "in_progress">,
  string
> = {
  cancelled: "Cancelled",
  failed: "Failed",
  interrupted: "Interrupted",
  incomplete: "Incomplete",
  limited: "Limit reached",
  refused: "Refused",
};

const layouts = new WeakMap<
  Message,
  { columns: number; expandedTools: boolean; lines: TranscriptLine[] }
>();

export function transcriptLines(
  messages: Message[],
  columns: number,
  expandedTools = false,
): TranscriptLine[] {
  const lines: TranscriptLine[] = [];
  const width = Math.max(1, columns - 2);

  for (const message of messages) {
    const cached = layouts.get(message);
    if (cached?.columns === columns && cached.expandedTools === expandedTools) {
      for (const line of cached.lines) lines.push(line);
      continue;
    }
    const start = lines.length;
    const append = (
      part: string,
      content: string,
      tone: TranscriptTone,
      kind: TranscriptLine["kind"],
      call?: ToolCall,
      path?: string,
    ) => {
      const safe = content
        .replace(/[\u0000-\u0008\u000b-\u001f\u007f-\u009f]/g, "")
        .replace(/\t/g, "    ");
      const rows = wrapAnsi(safe, width, { hard: true, trim: false }).split(
        "\n",
      );
      const highlighted = call
        ? highlightCode(call, rows.join("\n"), path)
        : undefined;
      rows.forEach((content, index) => {
        lines.push({
          id: `${message.id}:${part}:${index}`,
          content,
          spans: highlighted?.[index],
          tone,
          kind,
        });
      });
    };
    const user = message.role === "user";
    const tone = user ? "user" : "assistant";
    if (
      !user &&
      message.status === "in_progress" &&
      !message.content &&
      !message.tools?.length &&
      !message.steering?.length
    )
      continue;
    append("top", "", tone, "padding");
    const appendProse = (part: string, content: string) => {
      const previous = lines.at(-1);
      if (!user && !(previous?.kind === "padding" && previous.tone === tone))
        append(`${part}:top`, "", tone, "padding");
      append(part, content, tone, "content");
      if (!user) append(`${part}:bottom`, "", tone, "padding");
    };
    let offset = 0;
    const bytes = new TextEncoder().encode(message.content);
    for (const call of message.tools ?? []) {
      const end = Math.max(offset, Math.min(bytes.length, call.textOffset));
      const prose = new TextDecoder().decode(bytes.subarray(offset, end));
      if (prose) appendProse(`body:${call.id}`, prose);
      offset = end;
      const toolTone: TranscriptTone =
        call.status === "succeeded"
          ? "toolSuccess"
          : ["failed", "invalid", "unknown"].includes(call.status)
            ? "toolError"
            : "toolPending";
      const presentation = toolPresentation(call);
      if (lines.at(-1)?.tone.startsWith("tool"))
        append(`${call.id}:gap`, "", "space", "padding");
      append(`${call.id}:top`, "", toolTone, "padding");
      append(`${call.id}:title`, presentation.title, toolTone, "heading");
      if (presentation.input)
        append(
          `${call.id}:input`,
          presentation.input.slice(0, 800),
          toolTone,
          "content",
        );
      const output = presentation.output.split("\n");
      const excerpt = expandedTools
        ? presentation.output
        : output.slice(0, 6).join("\n").slice(0, 3000);
      if (excerpt)
        append(
          `${call.id}:output`,
          excerpt,
          toolTone,
          "content",
          call,
          presentation.path,
        );
      if (
        !expandedTools &&
        (output.length > 6 || presentation.output.length > 3000)
      )
        append(
          `${call.id}:more`,
          "… Ctrl+O expands tool output",
          toolTone,
          "content",
        );
      if (call.truncated)
        append(
          `${call.id}:truncated`,
          call.artifact
            ? `Output truncated · ${call.artifact}`
            : "Output truncated",
          toolTone,
          "status",
        );
      append(`${call.id}:bottom`, "", toolTone, "padding");
    }
    const remainder = new TextDecoder().decode(bytes.subarray(offset));
    if (
      remainder ||
      (!message.tools?.length && message.status !== "in_progress")
    )
      appendProse("body", remainder || "(no text)");
    for (const instruction of message.steering ?? [])
      append(
        `steering:${instruction.id}`,
        `↳ ${instruction.text}`,
        "pending",
        "content",
      );
    if (
      !user &&
      message.status !== "completed" &&
      message.status !== "in_progress"
    ) {
      const label = statusLabels[message.status];
      const detail =
        message.status === "limited"
          ? "turn budget exhausted; results saved"
          : message.status === "interrupted"
            ? "reply was not completed"
            : message.status !== "refused" && message.content
              ? "partial reply saved"
              : "";
      append(
        "status",
        detail ? `${label} · ${detail}` : label,
        message.status === "failed" ? "error" : "warning",
        "status",
      );
    }
    const previous = lines.at(-1);
    if (!(previous?.kind === "padding" && previous.tone === tone))
      append("bottom", "", tone, "padding");
    layouts.set(message, { columns, expandedTools, lines: lines.slice(start) });
  }
  return lines;
}
