export type McpDefinition = {
  name: string;
  enabled?: boolean;
  transport?:
    | { type: "stdio"; command: string; args?: string[] }
    | { type: "http"; url: string; headers?: Record<string, string> };
  environment?: Record<string, string>;
};

export type McpServer = {
  definition: McpDefinition;
  source: string;
  revision: string;
  status: string;
};

export type McpConfiguration = {
  revision: string;
  stale: boolean;
  error: string | null;
  servers: McpServer[];
  tools: { name: string; description: string; input_schema: unknown }[];
  registry_metadata: unknown;
};

export type McpOperation =
  | { action: "read" | "reload" }
  | { action: "save"; expected_revision: string; definition: McpDefinition }
  | { action: "remove"; expected_revision: string; name: string }
  | { action: "connect"; name: string; revision: string }
  | { action: "disconnect" | "discover"; name: string }
  | { action: "registry"; query: string };
