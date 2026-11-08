import { useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import type { Conversation } from "../types/conversation.js";
import type { Setup } from "../types/setup.js";

export function useSetup(
  agent: Agent,
  conversation: Conversation | null,
  initial: Setup,
  reconcile: (updated: Conversation) => void,
) {
  const [view, setView] = useState<Setup>(initial);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const epoch = useRef(0);
  const mounted = useRef(true);
  const read = useRef<AbortController | null>(null);
  const committing = useRef(false);
  const id = conversation?.id;
  const target = useRef(id);
  target.current = id;
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      epoch.current++;
      read.current?.abort();
    };
  }, []);
  useEffect(() => {
    target.current = id;
    epoch.current++;
    read.current?.abort();
    setOpen(false);
    setLoading(false);
  }, [id]);
  function close() {
    epoch.current++;
    read.current?.abort();
    setLoading(false);
    setOpen(false);
  }
  async function inspect() {
    read.current?.abort();
    const sequence = ++epoch.current;
    const controller = new AbortController();
    read.current = controller;
    setOpen(true);
    setLoading(true);
    setError(null);
    try {
      const result = await agent.readSetup(id, controller.signal);
      if (mounted.current && sequence === epoch.current) setView(result);
    } catch (error) {
      if (mounted.current && sequence === epoch.current)
        setError(errorMessage(error));
    } finally {
      if (mounted.current && sequence === epoch.current) {
        setLoading(false);
        read.current = null;
      }
    }
  }
  async function update(proposal: string, confirm: boolean) {
    if (!id || view.revision === null || committing.current) return;
    committing.current = true;
    setSaving(true);
    setError(null);
    const sequence = epoch.current;
    try {
      const updated = await agent.updateSetup(
        id,
        view.revision,
        proposal,
        confirm,
      );
      if (mounted.current && target.current === id) reconcile(updated);
      if (
        mounted.current &&
        target.current === id &&
        epoch.current === sequence
      )
        await inspect();
    } catch (error) {
      if (
        mounted.current &&
        target.current === id &&
        epoch.current === sequence
      )
        setError(errorMessage(error));
    } finally {
      committing.current = false;
      if (mounted.current) setSaving(false);
    }
  }
  return { view, open, loading, saving, error, inspect, update, close };
}
