import type { Conversation, Message, Run } from "../types/conversation.js";
import { messageStatuses } from "../types/conversation.js";
import type { ModelEffort, ModelSelection } from "../types/model.js";
import { modelEfforts } from "../types/model.js";
import { PresentationError } from "./errors.js";
import { parseModels } from "./models.js";
import { parseTool } from "./tools.js";
import { parseUsage } from "./usage.js";

const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function isConversationId(value: string): boolean {
  return uuid.test(value);
}

function id(value: unknown): string {
  if (typeof value !== "string" || !isConversationId(value))
    throw new PresentationError(
      "Invalid conversation/message identity returned by the agent.",
    );
  return value;
}

function effort(value: unknown): ModelEffort | null {
  if (value === null) return null;
  const effort = modelEfforts.find((effort) => effort === value);
  if (!effort)
    throw new PresentationError(
      "Invalid conversation effort returned by the agent.",
    );
  return effort;
}

function counter(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0)
    throw new PresentationError(
      "Invalid conversation metadata returned by the agent.",
    );
  return value;
}

export function parseConversation(value: unknown): Conversation {
  if (!record(value) || !Array.isArray(value.messages))
    throw new PresentationError("The agent did not return a conversation.");
  const model = parseModels({ models: [value.model] })[0];
  if (!model) throw new PresentationError("The conversation has no model.");
  const selectedEffort = effort(value.effort);
  if (selectedEffort && !model.efforts.includes(selectedEffort))
    throw new PresentationError("The conversation has an unsupported effort.");
  const ids = new Set<string>();
  const messages: Message[] = value.messages.map(
    (value: unknown, index: number) => {
      if (
        !record(value) ||
        typeof value.content !== "string" ||
        typeof value.modelId !== "string" ||
        value.role !== (index % 2 === 0 ? "user" : "assistant")
      )
        throw new PresentationError(
          "Invalid conversation message returned by the agent.",
        );
      const messageId = id(value.id);
      if (ids.has(messageId))
        throw new PresentationError(
          "Duplicate message identity returned by the agent.",
        );
      ids.add(messageId);
      const status = messageStatuses.find((status) => status === value.status);
      if (!status)
        throw new PresentationError(
          "Invalid conversation message status returned by the agent.",
        );
      return {
        id: messageId,
        role: index % 2 === 0 ? "user" : "assistant",
        content: value.content,
        status,
        modelId: value.modelId,
        effort: effort(value.effort),
        tools:
          value.tools === undefined
            ? []
            : (() => {
                if (!Array.isArray(value.tools))
                  throw new PresentationError("Invalid tool history.");
                const calls = value.tools.map(parseTool);
                const ids = new Set(calls.map((c) => c.id));
                if (
                  ids.size !== calls.length ||
                  calls.some(
                    (c) =>
                      c.messageId !== messageId ||
                      c.textOffset >
                        new TextEncoder().encode(value.content as string)
                          .length,
                  )
                )
                  throw new PresentationError(
                    "Invalid tool history identity or position.",
                  );
                return calls;
              })(),
        steering:
          value.steering === undefined
            ? []
            : (() => {
                if (!Array.isArray(value.steering))
                  throw new PresentationError("Invalid steering history.");
                return value.steering.map((s) => {
                  if (
                    !record(s) ||
                    typeof s.id !== "string" ||
                    !uuid.test(s.id) ||
                    typeof s.text !== "string" ||
                    typeof s.afterStep !== "number" ||
                    !Number.isSafeInteger(s.afterStep) ||
                    s.afterStep < 0
                  )
                    throw new PresentationError("Invalid steering history.");
                  return { id: s.id, text: s.text, afterStep: s.afterStep };
                });
              })(),
      };
    },
  );
  if (messages.length % 2 !== 0)
    throw new PresentationError(
      "Incomplete conversation returned by the agent.",
    );
  return {
    agent: value.agent == null ? null : parseAgent(value.agent),
    runs: value.runs == null ? [] : parseRuns(value.runs),
    resumeRequired: value.resumeRequired === true,
    selectedInstructions:
      typeof value.selectedInstructions === "string"
        ? value.selectedInstructions
        : null,
    id: id(value.id),
    workspace: typeof value.workspace === "string" ? value.workspace : null,
    usage: value.usage == null ? undefined : parseUsage(value.usage),
    providerConnectionId: id(value.providerConnectionId),
    model,
    effort: selectedEffort,
    messages,
    createdAt: counter(value.createdAt),
    updatedAt: counter(value.updatedAt),
    revision: counter(value.revision),
  };
}

function parseAgent(value: unknown): NonNullable<Conversation["agent"]> {
  if (
    !record(value) ||
    (value.role !== "griller" &&
      value.role !== "chief_of_staff" &&
      value.role !== "setup" &&
      value.role !== "research") ||
    (value.role === "chief_of_staff" || value.role === "research"
      ? typeof value.creatorId !== "string"
      : value.creatorId !== null)
  )
    throw new PresentationError("Invalid agent identity.");
  return {
    id: id(value.id),
    role: value.role,
    creatorId: value.creatorId === null ? null : id(value.creatorId),
  };
}

function parseRuns(value: unknown): Run[] {
  if (!Array.isArray(value))
    throw new PresentationError("Invalid run history.");
  return value.map((run: unknown) => {
    if (
      !record(run) ||
      typeof run.modelId !== "string" ||
      ![
        "grilling",
        "specification",
        "tickets",
        "coordination",
        "setup",
        "research",
      ].includes(String(run.task)) ||
      !["running", "needs_human", "stopped", "failed", "interrupted"].includes(
        String(run.status),
      )
    )
      throw new PresentationError("Invalid run.");
    return {
      id: id(run.id),
      agentId: id(run.agentId),
      replyId: id(run.replyId),
      modelId: run.modelId,
      effort: effort(run.effort),
      task: run.task as Run["task"],
      instructions:
        typeof run.instructions === "string" ? run.instructions : null,
      status: run.status as Run["status"],
    };
  });
}

export function selectionMetadata(selection: ModelSelection) {
  return {
    "harness/modelSelection": {
      providerId: selection.model.providerId,
      modelId: selection.model.id,
      effort: selection.effort,
    },
  };
}

export function parseArguments(args: string[]): string | undefined {
  if (!args.length) return;
  if (
    args.length !== 2 ||
    args[0] !== "--resume" ||
    !args[1] ||
    !isConversationId(args[1])
  )
    throw new PresentationError(
      "Usage: harness [--resume <conversation-id>] (a canonical lowercase UUID, not a path).",
    );
  return args[1];
}
