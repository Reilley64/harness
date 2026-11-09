import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import { PresentationError } from "../services/errors.js";
import type { Conversation, Message } from "../types/conversation.js";
import type { Guidance } from "../types/guidance.js";
import type { ModelSelection } from "../types/model.js";
import type { ToolCall } from "../types/tool.js";
import type { Usage } from "../types/usage.js";

type SendResult = { accepted: boolean; draft: string };
export function useConversation(
  agent: Agent,
  initial?: Conversation,
  onOpen?: (next: Conversation) => void,
) {
  const [conversation, setConversation] = useState<Conversation | null>(
    initial ?? null,
  );
  const current = useRef<Conversation | null>(initial ?? null);
  const [busy, setBusy] = useState(false);
  const [work, setWork] = useState<{
    startedAt: number;
    duration: number | null;
  } | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [usage, setUsage] = useState<Usage | null>(
    initial
      ? (initial.usage ?? null)
      : {
          inputTokens: 0,
          outputTokens: 0,
          contextTokens: null,
          modelId: null,
          partial: false,
        },
  );
  const active = useRef(false);
  const opening = useRef(0);
  const navigation = useRef<AbortController | null>(null);
  const owner = useRef(agent);
  const mounted = useRef(true);
  const restore = useRef(onOpen);
  owner.current = agent;
  restore.current = onOpen;
  const cancelOpen = useCallback(() => {
    opening.current++;
    navigation.current?.abort();
    navigation.current = null;
  }, []);
  useEffect(() => {
    owner.current = agent;
    mounted.current = true;
    return () => {
      mounted.current = false;
      cancelOpen();
    };
  }, [agent, cancelOpen]);
  const cancelled = useRef(false);
  const stopping = useRef<string | null>(null);
  const sessionReady = useRef<Promise<string | null>>(Promise.resolve(null));
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<{
    content: string;
    resume: boolean;
    reply: string;
    modelId: string;
    effort: ModelSelection["effort"];
    tools: ToolCall[];
    steering: { id: string; text: string; afterStep: number }[];
  } | null>(null);
  async function send(
    content: string,
    selection: ModelSelection | null,
    resume = false,
  ): Promise<SendResult> {
    if (stopping.current !== null && stopping.current === current.current?.id) {
      setError(
        "Wait for the selected execution's durable Stop acknowledgement.",
      );
      return { accepted: false, draft: content };
    }
    if (!resume && !content.trim()) return { accepted: false, draft: content };
    if (new TextEncoder().encode(content).length > 1_048_576) {
      setError("Prompt exceeds 1 MiB of UTF-8 text; your draft was preserved.");
      return { accepted: false, draft: content };
    }
    if (active.current) {
      try {
        const id = await sessionReady.current;
        if (!id || cancelled.current || !active.current)
          return { accepted: false, draft: content };
        const accepted = await agent.steer(id, content);
        if (accepted)
          setPending((p) =>
            p
              ? {
                  ...p,
                  steering: [
                    ...p.steering,
                    { id: crypto.randomUUID(), text: content, afterStep: 0 },
                  ],
                }
              : null,
          );
        else
          setError(
            "The turn has stopped or could not accept the instruction; your draft was preserved.",
          );
        return { accepted, draft: content };
      } catch (error) {
        setError(errorMessage(error));
        return { accepted: false, draft: content };
      }
    }
    if (!current.current && !selection) {
      setError(
        "Choose a model with /model, and sign in with /auth before sending.",
      );
      return { accepted: false, draft: content };
    }
    if (!resume && current.current?.resumeRequired) {
      setError(
        "Interrupted work is paused. Use /resume, or /resume inspect to acknowledge uncertain effects before a new instruction.",
      );
      return { accepted: false, draft: content };
    }
    cancelOpen();
    active.current = true;
    cancelled.current = false;
    let ready!: (id: string | null) => void;
    sessionReady.current = new Promise((resolve) => {
      ready = resolve;
    });
    const startedAt = performance.now();
    setBusy(true);
    setCancelling(false);
    setError(null);
    setWork({ startedAt, duration: null });
    let count = current.current?.messages.length ?? 0;
    let accepted = false;
    try {
      if (!current.current) {
        if (!selection)
          throw new PresentationError("Choose a model before sending.");
        current.current = await agent.startAgent(selection);
        setConversation(current.current);
      }
      count = current.current.messages.length;
      if (cancelled.current) return { accepted: false, draft: content };
      const prior = resume ? current.current.messages.at(-1) : undefined;
      setPending({
        content,
        resume,
        reply: prior?.content ?? "",
        modelId: selection?.model.id ?? current.current.model.id,
        effort: selection ? selection.effort : current.current.effort,
        tools: prior?.tools ?? [],
        steering: prior?.steering ?? [],
      });
      const reply = agent.createMessage(
        current.current.id,
        content,
        selection,
        (delta) =>
          setPending((p) => (p ? { ...p, reply: p.reply + delta } : null)),
        (call) =>
          setPending((p) =>
            p
              ? {
                  ...p,
                  tools: p.tools.some((c) => c.id === call.id)
                    ? p.tools.map((c) => (c.id === call.id ? call : c))
                    : [...p.tools, call],
                }
              : null,
          ),
        setUsage,
        resume,
      );
      ready(current.current.id);
      const updated = await reply;
      setUsage(updated.usage ?? null);
      current.current = updated;
      setConversation(updated);
      accepted = resume || updated.messages.length > count;
    } catch (error) {
      setError(errorMessage(error));
      if (current.current) {
        try {
          const restored = await agent.loadConversation(current.current.id);
          current.current = restored;
          setConversation(restored);
          setUsage(restored.usage ?? null);
          accepted = restored.messages.length > count;
        } catch {
          setError(
            errorMessage(error) +
              " Could not reload saved history; restart with --resume to check it before resending.",
          );
        }
      }
    } finally {
      ready(null);
      active.current = false;
      setWork({
        startedAt,
        duration: Math.max(0, performance.now() - startedAt),
      });
      setBusy(false);
      setCancelling(false);
      setPending(null);
    }
    return { accepted, draft: content };
  }
  function cancel() {
    const selected = current.current;
    if (!active.current && !selected) return;
    cancelled.current = true;
    setCancelling(true);
    if (!selected || stopping.current === selected.id) return;
    const id = selected.id;
    const navigationRevision = opening.current;
    stopping.current = id;
    const scoped = selected.agent && selected.agent.role !== "setup";
    const operation = scoped ? agent.stopExecution(id) : agent.cancelReply(id);
    void operation
      .catch(async (original) => {
        let failure = original;
        try {
          await agent.cancelReply(id);
        } catch (cleanup) {
          failure = new AggregateError(
            [original, cleanup],
            "Stop and local cancellation failed.",
          );
        }
        if (
          mounted.current &&
          owner.current === agent &&
          current.current?.id === id &&
          opening.current === navigationRevision
        )
          setError(errorMessage(failure));
      })
      .finally(() => {
        if (stopping.current === id) stopping.current = null;
        if (
          mounted.current &&
          owner.current === agent &&
          current.current?.id === id &&
          opening.current === navigationRevision &&
          !active.current
        )
          setCancelling(false);
      });
  }
  const messages: Message[] = pending
    ? [
        ...(pending.resume
          ? (conversation?.messages.slice(0, -1) ?? [])
          : (conversation?.messages ?? [])),
        ...(!pending.resume
          ? [
              {
                id: "pending-user",
                role: "user" as const,
                content: pending.content,
                status: "completed" as const,
                modelId: pending.modelId,
                effort: pending.effort,
              },
            ]
          : []),
        {
          id: "pending-assistant",
          role: "assistant",
          content: pending.reply,
          status: "in_progress",
          modelId: pending.modelId,
          effort: pending.effort,
          tools: pending.tools,
          steering: pending.steering,
        },
      ]
    : (conversation?.messages ?? []);
  return {
    conversation,
    cancelOpen,
    open: async (
      id?: string,
      role: "setup" | "griller" = "griller",
      selection: ModelSelection | null = null,
    ) => {
      if (active.current)
        throw new PresentationError(
          "Stop or finish the current run before opening another conversation.",
        );
      cancelOpen();
      const request = opening.current;
      const controller = new AbortController();
      navigation.current = controller;
      const owns = () =>
        mounted.current &&
        owner.current === agent &&
        request === opening.current &&
        !controller.signal.aborted &&
        !active.current;
      try {
        const existing = id
          ? null
          : await agent.readAgent(role, controller.signal);
        if (!owns()) return null;
        const target = id ?? existing?.id;
        const next = target
          ? await agent.loadConversation(target, controller.signal)
          : selection
            ? await agent.startAgent(selection, role)
            : null;
        if (!next)
          throw new PresentationError(
            "Choose a model before starting this role.",
          );
        if (!owns()) return null;
        current.current = next;
        setConversation(next);
        setUsage(next.usage ?? null);
        setError(null);
        setWork(null);
        restore.current?.(next);
        return next;
      } catch (error) {
        if (owns()) throw error;
        return null;
      } finally {
        if (navigation.current === controller) navigation.current = null;
      }
    },
    usage,
    messages,
    busy,
    cancelling,
    work,
    error,
    send,
    resume: () => send("", null, true),
    acknowledge: async () => {
      const target = current.current;
      if (!target || active.current) return;
      const request = opening.current;
      const owns = () =>
        mounted.current &&
        owner.current === agent &&
        request === opening.current &&
        current.current?.id === target.id &&
        !active.current;
      try {
        const updated = await agent.acknowledgeInterruption(target.id);
        if (
          owns() &&
          updated.id === target.id &&
          updated.revision >= (current.current?.revision ?? target.revision)
        ) {
          current.current = updated;
          setConversation(updated);
          setUsage(updated.usage ?? null);
          setError(null);
        }
      } catch (error) {
        if (owns() && current.current?.revision === target.revision)
          setError(errorMessage(error));
      }
    },
    adoptGuidance: async (
      guidance: Guidance,
      selection: ModelSelection | null,
    ) => {
      if (active.current)
        throw new PresentationError(
          "Stop or finish the run before adopting instructions.",
        );
      let target = current.current;
      if (!target) {
        if (!selection)
          throw new PresentationError(
            "Choose a model before creating the Griller.",
          );
        const created = await agent.startAgent(selection);
        const latest = current.current;
        if (
          latest &&
          (latest.id !== created.id || latest.revision > created.revision)
        )
          throw new PresentationError(
            "The conversation changed while creating the Griller; inspect again before adopting instructions.",
          );
        target = created;
        current.current = created;
        setConversation(created);
      }
      const id = target.id;
      const revision = guidance.revision ?? target.revision;
      const updated = await agent.adoptGuidance(id, revision, guidance);
      if (
        current.current?.id === id &&
        updated.id === id &&
        updated.revision >= current.current.revision
      ) {
        current.current = updated;
        setConversation(updated);
        setError(null);
      }
      return updated;
    },
    reconcile: (updated: Conversation) => {
      if (
        current.current?.id !== updated.id ||
        updated.revision < current.current.revision
      )
        return;
      current.current = updated;
      setConversation(updated);
      setUsage(updated.usage ?? null);
    },
    cancel,
  };
}
