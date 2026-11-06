import type { TranscriptSpan } from "./transcript-span.js";

export type TranscriptTone =
  | "user"
  | "assistant"
  | "pending"
  | "warning"
  | "error"
  | "space"
  | "toolPending"
  | "toolSuccess"
  | "toolError";

export type TranscriptLine = {
  id: string;
  content: string;
  spans?: TranscriptSpan[];
  tone: TranscriptTone;
  kind: "content" | "heading" | "status" | "padding";
};
