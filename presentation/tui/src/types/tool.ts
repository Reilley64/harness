export const toolStatuses = [
  "requested",
  "running",
  "succeeded",
  "failed",
  "cancelled",
  "invalid",
  "skipped",
  "interrupted",
  "unknown",
] as const;
export type ToolStatus = (typeof toolStatuses)[number];
export type ToolCall = {
  id: string;
  messageId: string;
  name: string;
  arguments: string;
  status: ToolStatus;
  output: string;
  artifact: string | null;
  truncated: boolean;
  elapsedMs: number;
  textOffset: number;
};
export type SteeringInstruction = {
  id: string;
  text: string;
  afterStep: number;
};
