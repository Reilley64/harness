export type Instruction = {
  id: string;
  version: string;
  source: string;
  upstream: string[];
  requires: string[];
  body: string;
};

export type InstructionSnapshot = {
  id: string;
  phase: string;
  scopes: string[];
  references: string[];
  instructions: Instruction[];
};

export type Guidance = {
  revision: number | null;
  selected: InstructionSnapshot | null;
  current: InstructionSnapshot | null;
  currentRun: string | null;
  historicalUnknown: string[];
  available: InstructionSnapshot | null;
  blocked: string[];
  overrides: string[];
  bundled: Instruction[];
};

export type GuidanceQuery = {
  phase?: string;
  scopes?: string[];
  references?: string[];
};
