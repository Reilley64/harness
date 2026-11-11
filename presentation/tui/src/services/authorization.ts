import open from "open";
import type {
  AuthorizationChallenge,
  ProviderConnection,
} from "../types/provider-connection.js";
import { PresentationError } from "./errors.js";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function validAuthorizationUrl(value: string): string {
  if (value.length > 65536)
    throw new PresentationError(
      "The authorization URL exceeds its size limit.",
    );
  const url = new URL(value);
  const loopback =
    url.hostname === "127.0.0.1" ||
    url.hostname === "[::1]" ||
    url.hostname === "localhost";
  if (
    !(url.protocol === "https:" || (url.protocol === "http:" && loopback)) ||
    url.username ||
    url.password ||
    url.href.includes("#")
  ) {
    throw new PresentationError(
      "The agent returned an unsafe authorization URL.",
    );
  }
  return url.toString();
}

export function parseAuthorizationChallenge(
  value: unknown,
): AuthorizationChallenge {
  if (
    !record(value) ||
    typeof value.authenticationId !== "string" ||
    typeof value.providerId !== "string" ||
    typeof value.authorizationUrl !== "string"
  ) {
    throw new PresentationError(
      "The agent returned an invalid authorization challenge.",
    );
  }
  return {
    authenticationId: value.authenticationId,
    providerId: value.providerId,
    authorizationUrl: validAuthorizationUrl(value.authorizationUrl),
  };
}

export function parseProviderConnection(value: unknown): ProviderConnection {
  if (
    !record(value) ||
    typeof value.id !== "string" ||
    typeof value.providerId !== "string"
  ) {
    throw new PresentationError(
      "The agent returned invalid provider connection information.",
    );
  }
  return {
    id: value.id,
    providerId: value.providerId,
  };
}

export async function openAuthorizationUrl(url: string): Promise<void> {
  await open(validAuthorizationUrl(url));
}
