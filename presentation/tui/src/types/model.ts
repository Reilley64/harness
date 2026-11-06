export const modelEfforts = ["low", "medium", "high", "extra_high"] as const;

export type ModelEffort = (typeof modelEfforts)[number];

export type ModelPricing = {
  input: number | null;
  cachedInput: number | null;
  output: number | null;
};

export type Model = {
  id: string;
  name: string;
  providerId: string;
  efforts: ModelEffort[];
  pricing: ModelPricing | null;
  contextWindow?: number | null;
};

export type ModelSelection = {
  model: Model;
  effort: ModelEffort | null;
};

export type ModelPickerState =
  | { status: "loading" }
  | { status: "failed"; message: string }
  | { status: "ready"; models: Model[] };
