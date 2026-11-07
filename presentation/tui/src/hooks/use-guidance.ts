import { useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import type { Conversation } from "../types/conversation.js";
import type { Guidance, GuidanceQuery } from "../types/guidance.js";

export function useGuidance(agent: Agent, conversation: Conversation | null) {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [guidance, setGuidance] = useState<Guidance | null>(null);
  const [error, setError] = useState<string | null>(null);
  const query = useRef<GuidanceQuery>({});
  const request = useRef(0);
  const intent = useRef(0);
  const controller = useRef<AbortController | null>(null);
  const completed = useRef<Guidance | null>(null);
  const id = conversation?.id;
  const target = useRef({ agent, id });
  const readTarget = useRef(id);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      request.current++;
      intent.current++;
      controller.current?.abort();
      controller.current = null;
    };
  }, []);

  useEffect(() => {
    const changingAgent = target.current.agent !== agent;
    target.current = { agent, id };
    if (!changingAgent && readTarget.current === id) return;
    request.current++;
    if (changingAgent) intent.current++;
    controller.current?.abort();
    controller.current = null;
    completed.current = null;
    query.current = {};
    readTarget.current = id;
    setOpen(false);
    setLoading(false);
    setGuidance(null);
    setError(null);
  }, [agent, id]);

  function cancelRead() {
    request.current++;
    controller.current?.abort();
    controller.current = null;
  }

  async function inspect(next: GuidanceQuery = query.current, sessionId = id) {
    cancelRead();
    intent.current++;
    const sequence = request.current;
    const pending = new AbortController();
    controller.current = pending;
    readTarget.current = sessionId;
    query.current = next;
    setOpen(true);
    setLoading(true);
    setGuidance(null);
    setError(null);
    try {
      const response = await agent.readGuidance(
        sessionId,
        next,
        pending.signal,
      );
      if (mounted.current && sequence === request.current) {
        completed.current = response;
        setGuidance(response);
      }
    } catch (error) {
      if (mounted.current && sequence === request.current)
        setError(errorMessage(error));
    } finally {
      if (mounted.current && sequence === request.current) {
        controller.current = null;
        setLoading(false);
      }
    }
  }

  async function adopt(inspected: Guidance, save: () => Promise<Conversation>) {
    const sequence = intent.current;
    const originalId = target.current.id;
    const updated = await save();
    if (
      !mounted.current ||
      sequence !== intent.current ||
      target.current.agent !== agent ||
      (target.current.id !== originalId && target.current.id !== updated.id)
    )
      return;
    await inspect(
      {
        phase: inspected.available?.phase,
        scopes: inspected.available?.scopes,
        references: inspected.available?.references,
      },
      updated.id,
    );
  }

  function close() {
    intent.current++;
    cancelRead();
    setLoading(false);
    setGuidance(completed.current);
    setOpen(false);
  }

  return { open, loading, guidance, error, inspect, adopt, close };
}
