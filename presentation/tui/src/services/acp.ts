import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { Writable } from "node:stream";
import { fileURLToPath } from "node:url";
import type { InitializeResponse } from "@agentclientprotocol/sdk";
import {
  client,
  methods,
  ndJsonStream,
  PROTOCOL_VERSION,
} from "@agentclientprotocol/sdk";
import type { Conversation } from "../types/conversation.js";
import type { Deciders, DecisionResponse } from "../types/decision.js";
import type { Guidance, GuidanceQuery } from "../types/guidance.js";
import type { McpConfiguration, McpOperation } from "../types/mcp.js";
import type { Model, ModelSelection } from "../types/model.js";
import type { Provider } from "../types/provider.js";
import type {
  AuthorizationChallenge,
  ProviderConnection,
} from "../types/provider-connection.js";
import type { ToolCall } from "../types/tool.js";
import type { Usage } from "../types/usage.js";
import type { ApprovedWork, WorkResponse } from "../types/work.js";
import {
  parseAuthorizationChallenge,
  parseProviderConnection,
  validAuthorizationUrl,
} from "./authorization.js";
import {
  parseChildInstruction,
  parseChildResult,
  parseChildren,
  parseExecution,
} from "./children.js";
import { parseConversation, selectionMetadata } from "./conversations.js";
import { parseDeciders, parseDecision } from "./decisions.js";
import { PresentationError } from "./errors.js";
import { parseGuidance } from "./guidance.js";
import { parseMcp } from "./mcp.js";
import { parseModelSelection, parseModels } from "./models.js";
import { readableStream } from "./readable-stream.js";
import { parseSetup } from "./setup.js";
import { parseTool } from "./tools.js";
import { parseUsageUpdate } from "./usage.js";
import { parseWork } from "./work.js";

export type Agent = ReturnType<typeof connectAgent>;

export { errorMessage } from "./errors.js";

function parseProviders(value: unknown): Provider[] {
  if (
    typeof value !== "object" ||
    value === null ||
    !("providers" in value) ||
    !Array.isArray(value.providers)
  ) {
    throw new PresentationError(
      "Invalid provider catalog returned by the ACP agent.",
    );
  }
  return value.providers.map((provider: unknown) => {
    if (
      typeof provider !== "object" ||
      provider === null ||
      !("id" in provider) ||
      typeof provider.id !== "string" ||
      !("name" in provider) ||
      typeof provider.name !== "string"
    ) {
      throw new PresentationError(
        "Invalid provider entry returned by the ACP agent.",
      );
    }
    return { id: provider.id, name: provider.name };
  });
}

export function connectAgent() {
  const workspace = process.env.INIT_CWD ?? process.cwd();
  const binary = fileURLToPath(
    new URL(
      process.platform === "win32"
        ? "../../../../target/debug/harness.exe"
        : "../../../../target/debug/harness",
      import.meta.url,
    ),
  );
  const child = spawn(binary, [], {
    cwd: workspace,
    stdio: ["pipe", "pipe", "pipe"],
  });
  let failure = "The ACP agent disconnected. Restart Harness to reconnect.";
  let closing = false;
  let initialization: Promise<InitializeResponse> | undefined;
  const challenges = new Map<
    string,
    { providerId: string; present: (challenge: AuthorizationChallenge) => void }
  >();
  const mcpChallenges = new Map<
    string,
    {
      workspace: string;
      revision: string;
      present: (url: string) => void;
    }
  >();
  const replies = new Map<
    string,
    {
      present: (delta: string) => void;
      tool?: (call: ToolCall) => void;
      usage?: (usage: Usage) => void;
      ready: Promise<boolean>;
      started: boolean;
      cancelled: boolean;
    }
  >();
  let steeringWrites: Promise<void> = Promise.resolve();
  let selectionWrites: Promise<void> = Promise.resolve();
  let pendingSelections = 0;
  const pendingReplies = new Set<Promise<Conversation>>();
  const pendingPolicyWrites = new Set<Promise<string[]>>();
  const pendingWorkWrites = new Set<Promise<WorkResponse>>();
  const pendingNative = new Set<Promise<unknown>>();
  const pendingChildren = new Set<Promise<unknown>>();
  const pendingMcp = new Set<Promise<McpConfiguration>>();
  const mcpControllers = new Set<AbortController>();
  const nativeControllers = new Map<string, AbortController>();
  const connection = client({ name: "harness-tui" })
    .onNotification(methods.client.session.update, ({ params }) => {
      const update = params.update;
      if (
        update.sessionUpdate === "agent_message_chunk" &&
        update.content.type === "text"
      )
        replies.get(params.sessionId)?.present(update.content.text);
      if (
        update.sessionUpdate === "tool_call" ||
        update.sessionUpdate === "tool_call_update"
      ) {
        const value = update._meta?.["harness/tool"];
        if (value !== undefined)
          replies.get(params.sessionId)?.tool?.(parseTool(value));
      }
    })
    .onNotification(
      "_harness/mcp/authorization",
      (value: unknown) => {
        if (
          typeof value !== "object" ||
          value === null ||
          !("authentication_id" in value) ||
          typeof value.authentication_id !== "string" ||
          !("workspace" in value) ||
          typeof value.workspace !== "string" ||
          !("revision" in value) ||
          typeof value.revision !== "string" ||
          !("authorization_url" in value) ||
          typeof value.authorization_url !== "string"
        )
          throw new PresentationError("Invalid MCP authorization challenge.");
        return {
          id: value.authentication_id,
          workspace: value.workspace,
          revision: value.revision,
          url: validAuthorizationUrl(value.authorization_url),
        };
      },
      ({ params }) => {
        const pending = mcpChallenges.get(params.id);
        if (
          pending &&
          pending.workspace === params.workspace &&
          pending.revision === params.revision
        )
          pending.present(params.url);
      },
    )
    .onNotification("_harness/usage", parseUsageUpdate, ({ params }) => {
      replies.get(params.sessionId)?.usage?.(params.usage);
    })
    .onNotification(
      "_harness/authentication",
      parseAuthorizationChallenge,
      ({ params: challenge }) => {
        const pending = challenges.get(challenge.authenticationId);
        if (pending && pending.providerId === challenge.providerId)
          pending.present(challenge);
      },
    )
    .connect(
      ndJsonStream(Writable.toWeb(child.stdin), readableStream(child.stdout)),
    );

  child.stderr.resume();
  child.once("error", () => {
    failure =
      "Could not start the ACP agent. Run mise run build:agent and try again.";
    connection.close(new PresentationError(failure));
  });
  child.once("exit", (code, signal) => {
    failure = `The ACP agent exited (${signal ?? code ?? "unknown status"}).`;
    connection.close(new PresentationError(failure));
  });

  function initialize(): Promise<InitializeResponse> {
    initialization ??= connection.agent
      .request(
        methods.agent.initialize,
        {
          protocolVersion: PROTOCOL_VERSION,
          clientCapabilities: {},
          clientInfo: { name: "harness-tui", version: "0.1.0" },
        },
        {
          cancellationSignal: AbortSignal.timeout(5000),
          stopWaitingOnCancel: true,
        },
      )
      .then((response) => {
        if (response.protocolVersion !== PROTOCOL_VERSION) {
          throw new PresentationError(
            `Unsupported ACP version: ${response.protocolVersion}`,
          );
        }
        if (response._meta?.["harness/providerCatalog"] !== true) {
          throw new PresentationError(
            "The ACP agent does not advertise the Harness provider catalog.",
          );
        }
        return response;
      });
    return initialization;
  }

  function updateMcp(
    operation: McpOperation,
    workspace: string,
    signal: AbortSignal,
    present?: (url: string) => void,
  ): Promise<McpConfiguration> {
    if (closing)
      return Promise.reject(new PresentationError("The agent is closing."));
    const controller = new AbortController();
    const cancellationSignal = AbortSignal.any([signal, controller.signal]);
    mcpControllers.add(controller);
    const pending = (async () => {
      cancellationSignal.throwIfAborted();
      const initialized = await initialize();
      cancellationSignal.throwIfAborted();
      if (initialized._meta?.["harness/mcp"] !== true)
        throw new PresentationError(
          "MCP configuration is not available from this agent.",
        );

      const authenticationId = randomUUID();
      if (operation.action === "connect" && present)
        mcpChallenges.set(authenticationId, {
          workspace,
          revision: operation.revision,
          present,
        });
      try {
        return parseMcp(
          await connection.agent.request<unknown>(
            "_harness/mcp",
            { workspace, authentication_id: authenticationId, ...operation },
            { cancellationSignal, stopWaitingOnCancel: false },
          ),
        );
      } finally {
        mcpChallenges.delete(authenticationId);
      }
    })();
    pendingMcp.add(pending);
    return pending.finally(() => {
      mcpControllers.delete(controller);
      pendingMcp.delete(pending);
    });
  }

  async function listProviders(signal: AbortSignal): Promise<Provider[]> {
    await initialize();
    signal.throwIfAborted();
    const response = await connection.agent.request<unknown>(
      "_harness/providers",
      {},
      {
        cancellationSignal: AbortSignal.any([
          signal,
          AbortSignal.timeout(15000),
        ]),
        stopWaitingOnCancel: true,
      },
    );
    return parseProviders(response);
  }

  async function listModels(signal: AbortSignal): Promise<Model[]> {
    const initialized = await initialize();
    signal.throwIfAborted();
    if (initialized._meta?.["harness/modelCatalog"] !== true)
      throw new PresentationError(
        "The ACP agent does not advertise the Harness model catalog.",
      );
    const response = await connection.agent.request<unknown>(
      "_harness/models",
      {},
      {
        cancellationSignal: AbortSignal.any([
          signal,
          AbortSignal.timeout(15000),
        ]),
        stopWaitingOnCancel: true,
      },
    );
    return parseModels(response);
  }

  function modelAllowlist(
    patterns?: string[],
    signal?: AbortSignal,
  ): Promise<string[]> {
    if (closing)
      return Promise.reject(new PresentationError("The agent is closing."));
    const cancellationSignal =
      patterns === undefined && signal
        ? AbortSignal.any([signal, AbortSignal.timeout(30000)])
        : AbortSignal.timeout(30000);
    const pending = (async () => {
      cancellationSignal.throwIfAborted();
      const initialized = await initialize();
      cancellationSignal.throwIfAborted();
      if (initialized._meta?.["harness/modelAllowlist"] !== true)
        throw new PresentationError(
          "The agent does not support model permission settings.",
        );
      const response = await connection.agent.request<unknown>(
        patterns === undefined
          ? "_harness/model-allowlist/read"
          : "_harness/model-allowlist/update",
        patterns === undefined ? {} : { patterns },
        {
          cancellationSignal,
          stopWaitingOnCancel: true,
        },
      );
      if (
        typeof response !== "object" ||
        response === null ||
        !("patterns" in response) ||
        !Array.isArray(response.patterns) ||
        response.patterns.some((value: unknown) => typeof value !== "string")
      )
        throw new PresentationError(
          "Invalid model allowlist returned by the agent.",
        );
      return response.patterns;
    })();
    if (patterns === undefined) return pending;
    pendingPolicyWrites.add(pending);
    return pending.finally(() => pendingPolicyWrites.delete(pending));
  }

  async function childRequest(
    method: string,
    params: Record<string, unknown>,
    signal?: AbortSignal,
    write = false,
  ): Promise<unknown> {
    const init = await initialize();
    signal?.throwIfAborted();
    if (closing || init._meta?.["harness/children"] !== true)
      throw new PresentationError(
        "Managed child controls unavailable or closing.",
      );
    const pending = connection.agent.request<unknown>(method, params, {
      cancellationSignal: signal,
    });
    if (write) pendingChildren.add(pending);
    try {
      return await pending;
    } finally {
      pendingChildren.delete(pending);
    }
  }

  async function readChildren(sessionId: string, signal?: AbortSignal) {
    return parseChildren(
      await childRequest("_harness/children/read", { sessionId }, signal),
    );
  }
  async function childResult(
    sessionId: string,
    executionId: string,
    signal?: AbortSignal,
  ) {
    return parseChildResult(
      await childRequest(
        "_harness/children/result",
        { sessionId, executionId },
        signal,
      ),
    );
  }
  async function controlChildren(
    sessionId: string,
    operation: "join" | "background" | "stop",
    args: Record<string, unknown>,
  ) {
    return parseChildren(
      await childRequest(
        `_harness/children/${operation}`,
        { ...args, sessionId },
        undefined,
        true,
      ),
    );
  }
  async function answerAgent(
    sessionId: string,
    requestRun: string,
    text: string,
  ) {
    await viewConversation(sessionId);
    return childRequest(
      "_harness/agent/answer",
      { sessionId, requestRun, text },
      undefined,
      true,
    );
  }

  async function steerChild(
    sessionId: string,
    executionId: string,
    id: string,
    text: string,
  ) {
    const value = await childRequest(
      "_harness/children/steer",
      { sessionId, executionId, id, text },
      undefined,
      true,
    );
    if (
      !value ||
      typeof value !== "object" ||
      !("instruction" in value) ||
      !("effectiveBoundary" in value) ||
      typeof value.effectiveBoundary !== "string"
    )
      throw new PresentationError(
        "Invalid durable child instruction acknowledgement.",
      );
    return {
      instruction: parseChildInstruction(value.instruction),
      effectiveBoundary: value.effectiveBoundary,
    };
  }

  async function dispatchChild(
    sessionId: string,
    args: Record<string, unknown>,
  ) {
    const state = await readChildren(sessionId);
    if (state?.sessionRevision === undefined)
      throw new PresentationError("Inspect an active parent before dispatch.");
    const value = await childRequest(
      "_harness/children/dispatch",
      { ...args, sessionId, revision: state.sessionRevision },
      undefined,
      true,
    );
    if (!value || typeof value !== "object" || !("execution" in value))
      throw new PresentationError("Invalid dispatch response.");
    return parseExecution(value.execution);
  }
  async function adoptResearch(
    sessionId: string,
    intent: string,
    revision: number,
    snapshot: string,
  ) {
    await childRequest(
      "_harness/children/guidance/adopt",
      { sessionId, intent, revision, snapshot },
      undefined,
      true,
    );
  }
  async function viewConversation(
    sessionId: string,
    signal?: AbortSignal,
  ): Promise<Conversation> {
    const value = await childRequest(
      "_harness/conversation/view",
      { sessionId },
      signal,
    );
    if (!value || typeof value !== "object" || !("conversation" in value))
      throw new PresentationError("Invalid actor snapshot.");
    const next = parseConversation(value.conversation);
    if (next.id !== sessionId)
      throw new PresentationError("Actor snapshot target changed.");
    return next;
  }

  async function nativeRequest(
    method: string,
    params: Record<string, unknown>,
    signal?: AbortSignal,
  ): Promise<unknown> {
    const init = await initialize();
    signal?.throwIfAborted();
    if (closing || init._meta?.["harness/nativeDecider"] !== true)
      throw new PresentationError("Native Decider unavailable or closing.");
    return connection.agent.request<unknown>(method, params, {
      cancellationSignal: signal,
    });
  }

  async function deciders(signal: AbortSignal): Promise<Deciders> {
    return parseDeciders(await nativeRequest("_harness/deciders", {}, signal));
  }

  async function chooseDecider(modelId: string): Promise<void> {
    const pending = nativeRequest("_harness/decider/update", { modelId });
    pendingNative.add(pending);
    try {
      parseDeciders(await pending);
    } finally {
      pendingNative.delete(pending);
    }
  }

  async function readDecision(
    id: string,
    signal: AbortSignal,
  ): Promise<DecisionResponse> {
    return parseDecision(
      await nativeRequest("_harness/decision/read", { id }, signal),
    );
  }

  async function selectDecision(id: string): Promise<DecisionResponse> {
    const controller = new AbortController();
    nativeControllers.set(id, controller);
    const pending = nativeRequest(
      "_harness/decision/select",
      { id },
      controller.signal,
    );
    pendingNative.add(pending);
    try {
      return parseDecision(await pending);
    } finally {
      pendingNative.delete(pending);
      if (nativeControllers.get(id) === controller)
        nativeControllers.delete(id);
    }
  }

  async function stopDecision(id: string): Promise<DecisionResponse> {
    return parseDecision(await nativeRequest("_harness/decision/stop", { id }));
  }

  async function readWork(
    id: string,
    signal: AbortSignal,
  ): Promise<WorkResponse> {
    const init = await initialize();
    signal.throwIfAborted();
    if (init._meta?.["harness/approvedWork"] !== true)
      throw new PresentationError("Approved work is unavailable.");
    return parseWork(
      await connection.agent.request<unknown>(
        "_harness/work/read",
        { id },
        { cancellationSignal: signal, stopWaitingOnCancel: true },
      ),
    );
  }
  function approveWork(
    work: ApprovedWork,
    conversation: Conversation,
    intent: string,
  ): Promise<WorkResponse> {
    if (closing)
      return Promise.reject(new PresentationError("The agent is closing."));
    const publication = work.publications.at(-1);
    if (!publication)
      return Promise.reject(
        new PresentationError("Published proposal is missing."),
      );
    const prior = work.approvals.find((a) => a.intent === intent);
    const pending = (async () => {
      await initialize();
      return parseWork(
        await connection.agent.request<unknown>("_harness/work/approve", {
          id: work.id,
          revision: prior?.revision ?? work.revision,
          version: prior?.version ?? publication.version,
          intent,
          sessionRevision: conversation.revision,
          providerId: conversation.model.providerId,
          modelId: conversation.model.id,
          effort: conversation.effort,
        }),
      );
    })();
    pendingWorkWrites.add(pending);
    return pending.finally(() => pendingWorkWrites.delete(pending));
  }

  async function createProviderConnection(
    providerId: string,
    present: (challenge: AuthorizationChallenge) => void,
    signal: AbortSignal,
  ): Promise<ProviderConnection> {
    const initialized = await initialize();
    signal.throwIfAborted();
    if (!initialized.authMethods?.some((method) => method.id === providerId)) {
      throw new PresentationError(
        `Authentication is not supported for provider ${providerId}.`,
      );
    }
    const authenticationId = randomUUID();
    challenges.set(authenticationId, { providerId, present });
    try {
      const response = await connection.agent.request(
        methods.agent.authenticate,
        {
          methodId: providerId,
          _meta: { "harness/authenticationId": authenticationId },
        },
        {
          cancellationSignal: AbortSignal.any([
            signal,
            AbortSignal.timeout(660000),
          ]),
          stopWaitingOnCancel: true,
        },
      );
      const created = parseProviderConnection(
        response._meta?.["harness/providerConnection"],
      );
      if (created.providerId !== providerId) {
        throw new PresentationError(
          "The agent returned a connection for a different provider.",
        );
      }
      return created;
    } finally {
      challenges.delete(authenticationId);
    }
  }

  async function readModelSelection(): Promise<ModelSelection | null> {
    await selectionWrites;
    const initialized = await initialize();
    if (initialized._meta?.["harness/modelSelection"] !== true)
      throw new PresentationError(
        "The agent does not support remembering model selection.",
      );
    const response = await connection.agent.request<unknown>(
      "_harness/model-selection/read",
      {},
      {
        cancellationSignal: AbortSignal.timeout(15000),
        stopWaitingOnCancel: true,
      },
    );
    return parseModelSelection(response);
  }

  function updateModelSelection(
    selection: ModelSelection,
  ): Promise<ModelSelection> {
    if (closing)
      return Promise.reject(new PresentationError("The agent is closing."));
    pendingSelections++;
    const pending = selectionWrites.then(async () => {
      const initialized = await initialize();
      if (initialized._meta?.["harness/modelSelection"] !== true)
        throw new PresentationError(
          "The agent does not support remembering model selection.",
        );
      const response = await connection.agent.request<unknown>(
        "_harness/model-selection/update",
        {
          providerId: selection.model.providerId,
          modelId: selection.model.id,
          effort: selection.effort,
        },
        {
          cancellationSignal: AbortSignal.timeout(30000),
          stopWaitingOnCancel: true,
        },
      );
      const saved = parseModelSelection(response);
      if (
        !saved ||
        saved.model.id !== selection.model.id ||
        saved.model.providerId !== selection.model.providerId ||
        saved.effort !== selection.effort
      )
        throw new PresentationError(
          "The agent saved a different model selection.",
        );
      return saved;
    });
    selectionWrites = pending.then(
      () => undefined,
      () => undefined,
    );
    return pending.finally(() => pendingSelections--);
  }

  async function readAgent(
    role?: "setup" | "griller",
    signal?: AbortSignal,
  ): Promise<Conversation | undefined> {
    if (closing) throw new PresentationError("The agent is closing.");
    const cancellationSignal = signal
      ? AbortSignal.any([signal, AbortSignal.timeout(15000)])
      : AbortSignal.timeout(15000);
    cancellationSignal.throwIfAborted();
    await initialize();
    cancellationSignal.throwIfAborted();
    if (closing) throw new PresentationError("The agent is closing.");
    const response = await connection.agent.request<{ conversation: unknown }>(
      "_harness/agent/read",
      { cwd: workspace, role },
      { cancellationSignal, stopWaitingOnCancel: true },
    );
    return response.conversation == null
      ? undefined
      : parseConversation(response.conversation);
  }

  async function startAgent(
    selection: ModelSelection,
    role?: "setup" | "griller",
  ): Promise<Conversation> {
    await initialize();
    await selectionWrites;
    const response = await connection.agent.request<{ conversation: unknown }>(
      "_harness/agent/start",
      { cwd: workspace, selection: selectionMetadata(selection), role },
    );
    return parseConversation(response.conversation);
  }

  async function readSetup(id?: string, signal?: AbortSignal) {
    if (closing) throw new PresentationError("The agent is closing.");
    signal?.throwIfAborted();
    await initialize();
    signal?.throwIfAborted();
    const response = await connection.agent.request<unknown>(
      "_harness/setup/read",
      { cwd: workspace, sessionId: id ?? null },
      { cancellationSignal: signal, stopWaitingOnCancel: true },
    );
    return parseSetup(response);
  }

  async function updateSetup(
    id: string,
    revision: number,
    proposal: string,
    confirm: boolean,
  ): Promise<Conversation> {
    if (closing) throw new PresentationError("The agent is closing.");
    const pending = connection.agent
      .request<{ conversation: unknown }>("_harness/setup/update", {
        sessionId: id,
        revision,
        proposal,
        confirm,
      })
      .then((response) => parseConversation(response.conversation));
    pendingReplies.add(pending);
    try {
      return await pending;
    } finally {
      pendingReplies.delete(pending);
    }
  }

  async function readGuidance(
    id?: string,
    query: GuidanceQuery = {},
    signal?: AbortSignal,
  ): Promise<Guidance> {
    if (closing) throw new PresentationError("The agent is closing.");
    const cancellationSignal = signal
      ? AbortSignal.any([signal, AbortSignal.timeout(15000)])
      : AbortSignal.timeout(15000);
    cancellationSignal.throwIfAborted();
    await initialize();
    cancellationSignal.throwIfAborted();
    if (closing) throw new PresentationError("The agent is closing.");
    const response = await connection.agent.request<unknown>(
      "_harness/guidance/read",
      { cwd: workspace, sessionId: id ?? null, ...query },
      { cancellationSignal, stopWaitingOnCancel: true },
    );
    return parseGuidance(response);
  }

  async function adoptGuidance(
    id: string,
    revision: number,
    guidance: Guidance,
  ): Promise<Conversation> {
    if (!guidance.available)
      throw new PresentationError("Required instructions are blocked.");
    const response = await connection.agent.request<{ conversation: unknown }>(
      "_harness/guidance/adopt",
      {
        sessionId: id,
        revision,
        available: guidance.available.id,
        phase: guidance.available.phase,
        scopes: guidance.available.scopes,
        references: guidance.available.references,
      },
    );
    return parseConversation(response.conversation);
  }

  async function acknowledgeInterruption(id: string): Promise<Conversation> {
    const response = await connection.agent.request<{ conversation: unknown }>(
      "_harness/agent/acknowledge",
      { sessionId: id },
    );
    return parseConversation(response.conversation);
  }

  async function createConversation(
    selection: ModelSelection,
  ): Promise<Conversation> {
    await initialize();
    await selectionWrites;
    const response = await connection.agent.request(
      methods.agent.session.new,
      { cwd: workspace, mcpServers: [], _meta: selectionMetadata(selection) },
      {
        cancellationSignal: AbortSignal.timeout(30000),
        stopWaitingOnCancel: true,
      },
    );
    const conversation = parseConversation(
      response._meta?.["harness/conversation"],
    );
    if (conversation.id !== response.sessionId)
      throw new PresentationError(
        "The agent returned a different conversation.",
      );
    return conversation;
  }

  async function loadConversation(
    id: string,
    signal?: AbortSignal,
  ): Promise<Conversation> {
    if (closing) throw new PresentationError("The agent is closing.");
    const cancellationSignal = signal
      ? AbortSignal.any([signal, AbortSignal.timeout(15000)])
      : AbortSignal.timeout(15000);
    cancellationSignal.throwIfAborted();
    await initialize();
    cancellationSignal.throwIfAborted();
    if (closing) throw new PresentationError("The agent is closing.");
    const response = await connection.agent.request(
      methods.agent.session.load,
      { sessionId: id, cwd: workspace, mcpServers: [] },
      { cancellationSignal, stopWaitingOnCancel: true },
    );
    const conversation = parseConversation(
      response?._meta?.["harness/conversation"],
    );
    if (conversation.id !== id)
      throw new PresentationError("The agent loaded a different conversation.");
    return conversation;
  }

  async function createMessage(
    id: string,
    content: string,
    selection: ModelSelection | null,
    present: (delta: string) => void,
    tool?: (call: ToolCall) => void,
    usage?: (usage: Usage) => void,
    resume = false,
  ): Promise<Conversation> {
    if (closing || replies.has(id))
      throw new PresentationError(
        "The conversation is unavailable or already replying.",
      );
    let ready!: (started: boolean) => void;
    const reply = {
      present,
      tool,
      usage,
      ready: new Promise<boolean>((resolve) => {
        ready = resolve;
      }),
      started: false,
      cancelled: false,
    };
    replies.set(id, reply);
    const pending = selectionWrites.then(async () => {
      if (reply.cancelled || closing) {
        ready(false);
        return loadConversation(id);
      }
      reply.started = true;
      const responsePromise = connection.agent.request<{
        _meta?: Record<string, unknown>;
      }>(
        resume ? "_harness/agent/resume" : methods.agent.session.prompt,
        resume
          ? { sessionId: id }
          : {
              sessionId: id,
              prompt: [{ type: "text", text: content }],
              ...(selection ? { _meta: selectionMetadata(selection) } : {}),
            },
      );
      ready(true);
      const response = await responsePromise;
      const conversation = parseConversation(
        response._meta?.["harness/conversation"],
      );
      if (conversation.id !== id)
        throw new PresentationError(
          "The agent replied to a different conversation.",
        );
      return conversation;
    });
    pendingReplies.add(pending);
    try {
      return await pending;
    } finally {
      ready(false);
      replies.delete(id);
      pendingReplies.delete(pending);
    }
  }

  function steer(id: string, text: string): Promise<boolean> {
    const instructionId = randomUUID();
    const pending = steeringWrites.then(async () => {
      const reply = replies.get(id);
      if (closing || !reply || reply.cancelled || !(await reply.ready))
        return false;
      const result = await connection.agent.request<unknown>(
        "_harness/conversation/steer",
        { sessionId: id, id: instructionId, text },
      );
      if (
        typeof result !== "object" ||
        result === null ||
        !("accepted" in result) ||
        typeof result.accepted !== "boolean"
      )
        throw new PresentationError("Invalid steering acknowledgement.");
      return result.accepted;
    });
    steeringWrites = pending.then(
      () => undefined,
      () => undefined,
    );
    return pending;
  }

  async function stopExecution(id: string): Promise<void> {
    const initialized = await initialize();
    if (initialized._meta?.["harness/executionStop"] !== true)
      throw new PresentationError("Durable execution Stop is unavailable.");
    const result = await childRequest(
      "_harness/execution/stop",
      { sessionId: id },
      undefined,
      true,
    );
    if (!parseChildren(result))
      throw new PresentationError(
        "Missing durable execution Stop acknowledgement.",
      );
  }

  async function cancelReply(id: string): Promise<void> {
    const reply = replies.get(id);
    if (reply) {
      reply.cancelled = true;
      if (!reply.started) return;
    }
    await connection.agent.notify(methods.agent.session.cancel, {
      sessionId: id,
    });
  }

  async function close(): Promise<void> {
    if (closing) return;
    closing = true;
    for (const controller of nativeControllers.values()) controller.abort();
    for (const controller of mcpControllers) controller.abort();
    if (
      replies.size ||
      pendingReplies.size ||
      pendingSelections ||
      pendingPolicyWrites.size ||
      pendingWorkWrites.size ||
      pendingNative.size ||
      pendingMcp.size ||
      pendingChildren.size
    ) {
      await Promise.allSettled([...replies.keys()].map(cancelReply));
      let timer: ReturnType<typeof setTimeout> | undefined;
      try {
        await Promise.race([
          Promise.allSettled([
            ...pendingReplies,
            selectionWrites,
            ...pendingPolicyWrites,
            ...pendingWorkWrites,
            ...pendingNative,
            ...pendingMcp,
            ...pendingChildren,
          ]),
          new Promise<void>((resolve) => {
            timer = setTimeout(resolve, 60000);
          }),
        ]);
      } finally {
        clearTimeout(timer);
      }
    }
    let drainError: unknown;
    try {
      if ((await initialize())._meta?.["harness/children"] === true)
        await connection.agent.request("_harness/children/drain", {});
    } catch (error) {
      drainError = error;
    }
    connection.close();
    child.stdin.end();
    if (!child.pid || child.exitCode !== null || child.signalCode !== null) {
      if (drainError) throw drainError;
      return;
    }
    const exited = new Promise<void>((resolve) =>
      child.once("close", () => resolve()),
    );
    const terminate = setTimeout(() => child.kill("SIGTERM"), 500);
    const kill = setTimeout(() => child.kill("SIGKILL"), 1500);
    try {
      await exited;
    } finally {
      clearTimeout(terminate);
      clearTimeout(kill);
    }
    if (drainError) throw drainError;
  }

  return {
    initialize,
    readChildren,
    childResult,
    controlChildren,
    dispatchChild,
    steerChild,
    answerAgent,
    adoptResearch,
    viewConversation,
    updateMcp,
    listProviders,
    listModels,
    modelAllowlist,
    readWork,
    approveWork,
    deciders,
    chooseDecider,
    readDecision,
    selectDecision,
    stopDecision,
    createProviderConnection,
    readModelSelection,
    updateModelSelection,
    readSetup,
    updateSetup,
    readAgent,
    startAgent,
    acknowledgeInterruption,
    readGuidance,
    adoptGuidance,
    createConversation,
    loadConversation,
    createMessage,
    workspace,
    stopExecution,
    cancelReply,
    steer,
    close,
    closed: connection.closed,
    failure: () => failure,
  };
}
