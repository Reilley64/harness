import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import type { Deciders, DecisionResponse } from "../types/decision.js";

export function useNativeDecider(
  agent: Agent,
  conversationId: string | undefined,
) {
  const [mode, setMode] = useState<"picker" | "inspector" | null>(null);
  const [settings, setSettings] = useState<Deciders | null>(null);
  const [view, setView] = useState<DecisionResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [pendingIntents, setPendingIntents] = useState<Set<string>>(
    () => new Set(),
  );
  const inFlight = useRef(new Set<string>());
  const owner = useRef(agent);
  const [error, setError] = useState<string | null>(null);
  const sequence = useRef(0),
    reading = useRef<AbortController | null>(null);
  const target = useRef<string | null>(null),
    mounted = useRef(true);
  const close = useCallback(() => {
    sequence.current++;
    reading.current?.abort();
    target.current = null;
    setMode(null);
    setLoading(false);
  }, []);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      sequence.current++;
      reading.current?.abort();
    };
  }, []);

  useEffect(() => {
    if (owner.current !== agent) {
      owner.current = agent;
      inFlight.current.clear();
      setPendingIntents(new Set());
    }
    void conversationId;
    close();
  }, [agent, conversationId, close]);

  function receive(id: string, response: DecisionResponse) {
    if (!mounted.current || owner.current !== agent || target.current !== id)
      return;
    setView((previous) =>
      previous?.decision?.id === id &&
      response.decision &&
      previous.decision.revision > response.decision.revision
        ? previous
        : response,
    );
  }

  async function inspect(id?: string) {
    close();
    const current = sequence.current;
    const controller = new AbortController();
    reading.current = controller;
    target.current = id ?? null;
    setMode(id ? "inspector" : "picker");
    setLoading(true);
    setError(null);
    setView(null);
    try {
      if (id) {
        const response = await agent.readDecision(id, controller.signal);
        if (sequence.current === current) receive(id, response);
      } else {
        const response = await agent.deciders(controller.signal);
        if (mounted.current && sequence.current === current)
          setSettings(response);
      }
    } catch (e) {
      if (mounted.current && sequence.current === current)
        setError(errorMessage(e));
    } finally {
      if (mounted.current && sequence.current === current) setLoading(false);
    }
  }

  async function choose(modelId: string) {
    if (saving) return;
    const current = sequence.current;
    setSaving(true);
    setError(null);
    try {
      await agent.chooseDecider(modelId);
      if (mounted.current && sequence.current === current) close();
    } catch (e) {
      if (mounted.current && sequence.current === current)
        setError(errorMessage(e));
    } finally {
      if (mounted.current) setSaving(false);
    }
  }

  async function select() {
    const id = target.current;
    if (
      !id ||
      inFlight.current.has(id) ||
      loading ||
      view?.decision?.id !== id ||
      view.decision.phase !== "prepared"
    )
      return;

    inFlight.current.add(id);
    setPendingIntents(new Set(inFlight.current));
    setError(null);
    try {
      receive(id, await agent.selectDecision(id));
    } catch (e) {
      if (mounted.current && owner.current === agent && target.current === id)
        setError(errorMessage(e));
    } finally {
      if (mounted.current && owner.current === agent) {
        inFlight.current.delete(id);
        setPendingIntents(new Set(inFlight.current));
      }
    }
  }

  async function stop() {
    const id = target.current;
    if (!id) return;
    const current = sequence.current;
    try {
      const result = await agent.stopDecision(id);
      if (mounted.current && sequence.current === current) setView(result);
    } catch (e) {
      if (mounted.current && sequence.current === current)
        setError(errorMessage(e));
    }
  }

  const pending =
    pendingIntents.has(target.current ?? "") &&
    (!view?.decision ||
      view.decision.phase === "prepared" ||
      view.decision.phase === "pending");

  return {
    mode,
    settings,
    view,
    loading,
    saving,
    pending,
    error,
    inspect,
    choose,
    select,
    stop,
    close,
  };
}
