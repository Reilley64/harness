import type { Model } from "./model.js";

export type Decision = {
  id: string;
  revision: number;
  parentId: string;
  role: "research" | "implementer" | "reviewer";
  phase: string;
  details: string;
};

export type DecisionResponse = {
  decision: Decision | null;
  blocked: string | null;
};

export type Deciders = {
  models: Model[];
  selected: string | null;
};
