export type Usage = {
  inputTokens: number | null;
  outputTokens: number | null;
  contextTokens: number | null;
  modelId: string | null;
  partial: boolean;
};
