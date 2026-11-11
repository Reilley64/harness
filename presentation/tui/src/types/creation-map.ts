import type { ChildExecution } from "./children.js";

export type CreationNode = {
  id: string;
  conversationId: string;
  creatorId: string | null;
  role: string;

  revision: number;
  createdAt: number;
  runId: string | null;
  savedStatus: string | null;
  resumeRequired: boolean;

  coordination: {
    revision: number;
    status: string;
    gate: string;
    authorityComplete: boolean;
    restartRequired: boolean;
  } | null;
  execution: ChildExecution | null;
  pendingResults: number;
  requiredFollowUp: number;
};

export type CreationPage = {
  agents: CreationNode[];
  next: string | null;
};
