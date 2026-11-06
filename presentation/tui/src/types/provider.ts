export type Provider = {
  id: string;
  name: string;
};

export type ProviderPickerState =
  | { status: "loading" }
  | { status: "failed"; message: string }
  | { status: "ready"; providers: Provider[]; selected: number };
