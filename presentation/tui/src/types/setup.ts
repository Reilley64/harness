import type { ToolCall } from "./tool.js";

export type SetupProposal = ToolCall & {
  draft: {
    workspace: string;
    path: string;
    before: string | null;
    after: string;
    roots: [string, string | null][];
  };
};
export type Setup = {
  workspace: string;
  roots: string[];
  missing: string[];
  triage: boolean;
  revision: number | null;
  proposals: SetupProposal[];
};
