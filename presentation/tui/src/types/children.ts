import type { Conversation } from "./conversation.js";
import type { Guidance } from "./guidance.js";

export type ChildExecution = {
  intent: string;
  assignmentId: string;
  executionId: string;
  agentId: string;
  creatorId: string;
  parentConversationId: string;
  parentRunId: string;

  modelId: string;
  providerId: string;
  effort: string | null;
  instructions: string;
  status: string;
  stopRequested: boolean;

  outcomeId: string | null;
  deliveredTo: string | null;
  followUpCompletedBy: string | null;
  source: string;
};
export type CapturedJoin = {
  id: string;
  runId: string;
  mode: "any" | "all";
  executions: string[];
  backgrounded: boolean;
};
export type Children = {
  agentId: string;
  conversationId: string;
  revision: number;
  sessionRevision?: number;
  authorityComplete: boolean;
  contextBlocked: boolean | null;

  gate: string;
  status: string;
  settled: boolean;
  children: ChildExecution[];
  joins: CapturedJoin[];
};
export type ChildResult = {
  execution: ChildExecution;
  receipt: null | {
    id: string;
    status: string;
    conversationId: string;
    runId: string | null;
    replyId: string | null;
    untrustedSummary: string;
    artifacts: string[];
  };
  authority: string;
};
export type ChildInspection = {
  history: Conversation | null;
  result: ChildResult | null;
  guidance: Guidance | null;
};
