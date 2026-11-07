import type { Model, ModelEffort } from "./model.js";
import type { SteeringInstruction, ToolCall } from "./tool.js";
import type { Usage } from "./usage.js";

export const messageStatuses = [
  "in_progress",
  "completed",
  "cancelled",
  "failed",
  "interrupted",
  "incomplete",
  "limited",
  "refused",
] as const;

export type MessageStatus = (typeof messageStatuses)[number];

export type Message = {
  id: string;
  role: "user" | "assistant";
  content: string;
  status: MessageStatus;
  modelId: string;
  tools?: ToolCall[];
  steering?: SteeringInstruction[];
  effort: ModelEffort | null;
};

export type AgentIdentity = {
  id: string;
  role: "griller";
  creatorId: null;
};

export type Run = {
  id: string;
  agentId: string;
  replyId: string;
  modelId: string;
  effort: ModelEffort | null;
  task: "grilling";
  status: "running" | "needs_human" | "stopped" | "failed" | "interrupted";
};

export type Conversation = {
  agent?: AgentIdentity | null;
  runs?: Run[];
  resumeRequired?: boolean;
  workspace?: string | null;
  usage?: Usage;
  id: string;
  providerConnectionId: string;
  model: Model;
  effort: ModelEffort | null;
  messages: Message[];
  createdAt: number;
  updatedAt: number;
  revision: number;
};
