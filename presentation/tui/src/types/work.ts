export type WorkPublication = {
  intent: string;
  version: string;
  document: string;
};
export type WorkApproval = {
  intent: string;
  version: string;
  revision: number;
  providerId: string;
  modelId: string;
  effort: string | null;
  conversationId: string;
  approvedAt: number;
  delivered: boolean;
  current: boolean;
};
export type ApprovedWork = {
  id: string;
  creatorId: string;
  sessionId: string;
  workspace: string;
  revision: number;
  publications: WorkPublication[];
  approvals: WorkApproval[];
  unavailable: string[];
  complete: false;
};
export type WorkResponse = {
  work: ApprovedWork | null;
  conversationId: string | null;
};
