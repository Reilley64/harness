export type Usage = {
  inputTokens: number | null;
  outputTokens: number | null;
  contextTokens: number | null;
  modelId: string | null;
  partial: boolean;
  providerWait?: {
    kind: "transient" | "quota";
    failures: number;
    allowance: number;
    retryAt: number | null;
  } | null;
};
