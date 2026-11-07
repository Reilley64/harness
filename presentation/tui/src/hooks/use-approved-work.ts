import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import type { Conversation } from "../types/conversation.js";
import type { ApprovedWork } from "../types/work.js";
export function useApprovedWork(
  agent: Agent,
  conversation: Conversation | null,
) {
  const [open, setOpen] = useState(false);
  const [work, setWork] = useState<ApprovedWork | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(false);
  const [coordinator, setCoordinator] = useState<string | null>(null);
  const request = useRef(0),
    controller = useRef<AbortController | null>(null),
    mounted = useRef(true);
  const target = useRef(conversation?.id);
  target.current = conversation?.id;
  const intent = useRef<string | null>(null);
  const last = useRef<{
    id: string;
    work: ApprovedWork;
    conversation: Conversation;
    intent: string;
  } | null>(null);
  const close = useCallback(() => {
    request.current++;
    controller.current?.abort();
    setOpen(false);
    setLoading(false);
  }, []);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      request.current++;
      controller.current?.abort();
    };
  }, []);
  const owner = useRef({ agent, id: conversation?.id });
  useEffect(() => {
    owner.current = { agent, id: conversation?.id };
    close();
    setWork(null);
    setCoordinator(null);
    last.current = null;
    intent.current = null;
  }, [agent, conversation?.id, close]);
  async function inspect(id: string) {
    close();
    const sequence = request.current;
    const c = new AbortController();
    controller.current = c;
    setOpen(true);
    setLoading(true);
    setError(null);
    setCoordinator(null);
    try {
      const r = await agent.readWork(id, c.signal);
      if (mounted.current && sequence === request.current) {
        setWork(r.work);
        intent.current =
          r.work?.approvals.find((a) => a.current)?.intent ??
          crypto.randomUUID();
        last.current = null;
        if (!r.work) setError("Published work was not found.");
      }
    } catch (e) {
      if (mounted.current && sequence === request.current)
        setError(errorMessage(e));
    } finally {
      if (mounted.current && sequence === request.current) setLoading(false);
    }
  }
  async function approve() {
    if (!work || !conversation || saving || loading) return;
    if (work.sessionId !== conversation.id) {
      setError(
        "Open this work's Griller before approving its exact selection.",
      );
      return;
    }
    const sequence = request.current,
      id = conversation.id;
    const payload = last.current ?? {
      id: work.id,
      work,
      conversation,
      intent: intent.current ?? crypto.randomUUID(),
    };
    last.current = payload;
    setSaving(true);
    setError(null);
    try {
      const r = await agent.approveWork(
        payload.work,
        payload.conversation,
        payload.intent,
      );
      if (
        mounted.current &&
        sequence === request.current &&
        target.current === id
      ) {
        setWork(r.work);
        setCoordinator(r.conversationId);
      }
    } catch (e) {
      if (
        mounted.current &&
        sequence === request.current &&
        target.current === id
      )
        setError(
          errorMessage(e) +
            " Reload work to reconcile a possibly committed approval; retry uses the original intent.",
        );
    } finally {
      if (mounted.current) setSaving(false);
    }
  }
  return {
    open,
    work,
    error,
    saving,
    loading,
    coordinator,
    inspect,
    approve,
    close,
  };
}
