import { useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import { PresentationError } from "../services/errors.js";
import type { Conversation, Message } from "../types/conversation.js";
import type { ModelSelection } from "../types/model.js";
import type { ToolCall } from "../types/tool.js";
import type { Usage } from "../types/usage.js";

type SendResult = { accepted: boolean; draft: string };
export function useConversation(agent: Agent, initial?: Conversation) {
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
  const cancelled = useRef(false);
  const sessionReady = useRef<Promise<string | null>>(Promise.resolve(null));
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<{
    content: string;
    reply: string;
    modelId: string;
    effort: ModelSelection["effort"];
    tools: ToolCall[];
    steering: { id: string; text: string; afterStep: number }[];
  } | null>(null);
  async function send(
    content: string,
    selection: ModelSelection | null,
  ): Promise<SendResult> {
    if (!content.trim()) return { accepted: false, draft: content };
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
    const count = current.current?.messages.length ?? 0;
    let accepted = false;
    try {
      if (!current.current) {
        if (!selection)
          throw new PresentationError("Choose a model before sending.");
        current.current = await agent.createConversation(selection);
        setConversation(current.current);
      }
      if (cancelled.current) return { accepted: false, draft: content };
      setPending({
        content,
        reply: "",
        modelId: selection?.model.id ?? current.current.model.id,
        effort: selection ? selection.effort : current.current.effort,
        tools: [],
        steering: [],
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
      );
      ready(current.current.id);
      const updated = await reply;
      setUsage(updated.usage ?? null);
      current.current = updated;
      setConversation(updated);
      accepted = updated.messages.length > count;
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
    if (!active.current) return;
    cancelled.current = true;
    setCancelling(true);
    if (current.current)
      void agent
        .cancelReply(current.current.id)
        .catch((error) => setError(errorMessage(error)));
  }
  const messages: Message[] = pending
    ? [
        ...(conversation?.messages ?? []),
        {
          id: "pending-user",
          role: "user",
          content: pending.content,
          status: "completed",
          modelId: pending.modelId,
          effort: pending.effort,
        },
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
    usage,
    messages,
    busy,
    cancelling,
    work,
    error,
    send,
    cancel,
  };
}
