import type { McpConfiguration, McpDefinition } from "../types/mcp.js";
import { PresentationError } from "./errors.js";

export function parseMcpDefinition(value: unknown): McpDefinition {
  if (
    typeof value !== "object" ||
    value === null ||
    !("name" in value) ||
    typeof value.name !== "string"
  )
    throw new PresentationError(
      "MCP definition requires a stable name. Use nonsecret JSON only.",
    );
  if (
    "environment" in value &&
    (typeof value.environment !== "object" ||
      value.environment === null ||
      Object.values(value.environment).some(
        (reference) =>
          typeof reference !== "string" ||
          !reference.startsWith("HARNESS_MCP_"),
      ))
  )
    throw new PresentationError(
      "Environment values must be provisioned HARNESS_MCP_ references, not secrets.",
    );

  return value as McpDefinition;
}

export function parseMcp(value: unknown): McpConfiguration {
  if (
    typeof value !== "object" ||
    value === null ||
    !("revision" in value) ||
    typeof value.revision !== "string" ||
    !("stale" in value) ||
    typeof value.stale !== "boolean" ||
    !("servers" in value) ||
    !Array.isArray(value.servers) ||
    !("tools" in value) ||
    !Array.isArray(value.tools)
  )
    throw new PresentationError("Invalid MCP configuration response.");

  for (const server of value.servers) {
    if (
      typeof server !== "object" ||
      server === null ||
      typeof server.source !== "string" ||
      typeof server.revision !== "string" ||
      typeof server.status !== "string"
    )
      throw new PresentationError("Invalid MCP server status.");
    parseMcpDefinition(server.definition);
  }

  return value as McpConfiguration;
}
