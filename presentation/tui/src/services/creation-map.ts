import type { CreationNode, CreationPage } from "../types/creation-map.js";
import { parseExecution } from "./children.js";
import { isConversationId } from "./conversations.js";
import { PresentationError } from "./errors.js";

function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new PresentationError("Invalid creation-map state.");
  return value as Record<string, unknown>;
}

function identity(value: unknown): string {
  if (typeof value !== "string" || !isConversationId(value))
    throw new PresentationError("Invalid creation-map identity.");
  return value;
}

function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0)
    throw new PresentationError("Invalid creation-map revision/count.");
  return value;
}

function flag(value: unknown): boolean {
  if (typeof value !== "boolean")
    throw new PresentationError("Invalid creation-map authority flag.");
  return value;
}

function state(value: unknown): string {
  if (
    typeof value !== "string" ||
    ![
      "open",
      "ready",
      "held",
      "running",
      "waiting",
      "needs_human",
      "approval",
      "stopped",
      "resume_required",
      "unknown",
      "failed",
      "completed",
      "interrupted",
    ].includes(value)
  )
    throw new PresentationError("Unsupported creation-map status/gate.");
  return value;
}

export function parseCreationPage(value: unknown): CreationPage {
  const page = record(value);
  if (!Array.isArray(page.agents) || page.agents.length > 64)
    throw new PresentationError("Invalid creation-map page.");
  const seen = new Set<string>();
  const agents: CreationNode[] = page.agents.map((value) => {
    const node = record(value);
    const id = identity(node.id);
    if (seen.has(id))
      throw new PresentationError("Duplicate creation-map agent.");
    seen.add(id);
    if (
      typeof node.role !== "string" ||
      ![
        "setup",
        "griller",
        "chief_of_staff",
        "research",
        "decider",
        "implementer",
        "reviewer",
        "retro",
      ].includes(node.role)
    )
      throw new PresentationError("Unsupported creation-map role.");
    const coordination =
      node.coordination === null ? null : record(node.coordination);
    return {
      id,
      conversationId: identity(node.conversationId),
      creatorId: node.creatorId === null ? null : identity(node.creatorId),
      role: node.role,

      revision: integer(node.revision),
      createdAt: integer(node.createdAt),
      runId: node.runId === null ? null : identity(node.runId),
      savedStatus: node.savedStatus === null ? null : state(node.savedStatus),
      resumeRequired: flag(node.resumeRequired),

      coordination:
        coordination === null
          ? null
          : {
              revision: integer(coordination.revision),
              status: state(coordination.status),
              gate: state(coordination.gate),
              authorityComplete: flag(coordination.authorityComplete),
              restartRequired: flag(coordination.restartRequired),
            },
      execution:
        node.execution === null ? null : parseExecution(node.execution),
      pendingResults: integer(node.pendingResults),
      requiredFollowUp: integer(node.requiredFollowUp),
    };
  });
  return { agents, next: page.next === null ? null : identity(page.next) };
}

export function agentLabel(role: string): string {
  switch (role) {
    case "setup":
      return "Setup";
    case "griller":
      return "Griller";
    case "chief_of_staff":
      return "Chief of Staff";
    case "research":
      return "Research";
    case "decider":
      return "Decider";
    case "implementer":
      return "Implementer";
    case "reviewer":
      return "Reviewer";
    case "retro":
      return "Retro";
    default:
      return "Conversation";
  }
}

export function creationStatus(node: CreationNode): string {
  if (node.coordination && !node.coordination.authorityComplete)
    return "authority unavailable";
  if (
    node.resumeRequired ||
    node.execution?.resumeRequired ||
    node.coordination?.restartRequired
  )
    return "Resume required";
  if (
    node.execution &&
    ["completed", "failed", "stopped"].includes(node.execution.status)
  )
    return node.execution.status;
  if (node.execution?.stopRequested) return "Stop requested";
  if (node.coordination && node.coordination.gate !== "open")
    return node.coordination.status;
  return (
    node.execution?.status ??
    node.coordination?.status ??
    (node.savedStatus ? `saved ${node.savedStatus}` : "no saved run")
  );
}
