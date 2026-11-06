import type { Provider } from "./provider.js";

export type ProviderConnection = {
  id: string;
  providerId: string;
};

export type AuthorizationChallenge = {
  authenticationId: string;
  providerId: string;
  authorizationUrl: string;
};

export type ProviderConnectionState =
  | { status: "starting"; provider: Provider }
  | {
      status: "authorizing";
      provider: Provider;
      authorizationUrl: string;
      browserError?: string;
    }
  | { status: "failed"; provider: Provider; message: string }
  | { status: "connected"; provider: Provider; connection: ProviderConnection };
