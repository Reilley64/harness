import type { Key } from "ink";
import { usePaste } from "ink";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/errors.js";
import type { ChildResult, Children } from "../types/children.js";
import type { Conversation } from "../types/conversation.js";
import type { Guidance } from "../types/guidance.js";

export function useChildren(agent: Agent, parentId?: string) {
  const [visible, setVisible] = useState(false);
  const [state, setState] = useState<Children | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [members, setMembers] = useState(new Set<string>());
  const [join, setJoin] = useState(0);
  const [history, setHistory] = useState<Conversation | null>(null);
  const [result, setResult] = useState<ChildResult | null>(null);
  const [guidance, setGuidance] = useState<Guidance | null>(null);
  const [intent, setIntent] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const owner = useRef(parentId);
  const controller = useRef<AbortController | null>(null);
  usePaste(() => undefined, { isActive: visible });
  const target = useRef(parentId);
  target.current = parentId;

  const close = useCallback(() => {
    generation.current++;
    controller.current?.abort();
    setVisible(false);
    setHistory(null);
    setResult(null);
    setGuidance(null);
    setConfirmation(false);
  }, []);

  useEffect(
    () => () => {
      generation.current++;
      controller.current?.abort();
    },
    [],
  );

  useEffect(() => {
    if (owner.current !== parentId) {
      close();
      owner.current = parentId;
      setState(null);
      setSelected(null);
      setMembers(new Set());
    }
  }, [parentId, close]);

  async function run(
    action: (id: string, signal: AbortSignal) => Promise<void>,
  ) {
    const id = target.current;
    if (!id) {
      setError("Open the actual parent conversation first.");
      setVisible(true);
      return;
    }
    const ticket = ++generation.current;
    controller.current?.abort();
    const abort = new AbortController();
    controller.current = abort;
    setBusy(true);
    setError(null);
    try {
      await action(id, abort.signal);
      return generation.current === ticket && !abort.signal.aborted;
    } catch (error) {
      if (generation.current === ticket) setError(errorMessage(error));
      return false;
    } finally {
      if (generation.current === ticket) setBusy(false);
    }
  }
  async function refresh() {
    await run(async (id, signal) => {
      const snapshot = await agent.readChildren(id, signal);
      if (signal.aborted || target.current !== id) return;
      setState(snapshot);
      setSelected((current) =>
        snapshot?.children.some((c) => c.executionId === current)
          ? current
          : (snapshot?.children[0]?.executionId ?? null),
      );
    });
  }
  async function inspect() {
    setVisible(true);
    setHistory(null);
    setResult(null);
    setGuidance(null);
    await refresh();
  }
  async function inspectGuidance(key: string) {
    setVisible(true);
    setIntent(key);
    setHistory(null);
    setResult(null);
    await run(async (id, signal) => {
      const view = await agent.readGuidance(
        id,
        { phase: "research", scopes: [], references: [] },
        signal,
      );
      if (!signal.aborted && target.current === id) setGuidance(view);
    });
  }
  async function dispatch(args: Record<string, unknown>) {
    setVisible(true);
    if (
      await run(async (id) => {
        await agent.dispatchChild(id, args);
      })
    )
      await refresh();
  }
  const child = state?.children.find((c) => c.executionId === selected);
  const captured = state?.joins[join % Math.max(1, state.joins.length)];
  async function control(
    operation: "join" | "background" | "stop",
    args: Record<string, unknown>,
  ) {
    await run(async (id, signal) => {
      const snapshot = await agent.controlChildren(id, operation, args);
      if (!signal.aborted && target.current === id)
        setState((current) =>
          !current || !snapshot || snapshot.revision >= current.revision
            ? snapshot
            : current,
        );
    });
  }
  async function openHistory() {
    if (!child) return;
    const expected = child.agentId;
    await run(async (_id, signal) => {
      const view = await agent.viewConversation(expected, signal);
      if (!signal.aborted) {
        setHistory(view);
        setResult(null);
      }
    });
  }
  async function openResult() {
    if (!child) return;
    const execution = child.executionId;
    await run(async (id, signal) => {
      const view = await agent.childResult(id, execution, signal);
      if (!signal.aborted) {
        setResult(view);
        setHistory(null);
      }
    });
  }
  const historyId = history?.id;
  useEffect(() => {
    if (!visible || !historyId) return;
    const id = historyId;
    let active = true;
    const abort = new AbortController();
    const timer = setInterval(() => {
      void agent
        .viewConversation(id, abort.signal)
        .then((view) => {
          if (active)
            setHistory((current) =>
              current?.id === id && view.revision >= current.revision
                ? view
                : current,
            );
        })
        .catch((error) => {
          if (active) setError(errorMessage(error));
        });
    }, 750);
    return () => {
      active = false;
      abort.abort();
      clearInterval(timer);
    };
  }, [agent, visible, historyId]);
  function input(value: string, key: Key) {
    if (key.eventType === "release" || key.eventType === "repeat") return;
    if (key.escape) {
      if (confirmation) setConfirmation(false);
      else if (history || result || guidance) {
        setHistory(null);
        setResult(null);
        setGuidance(null);
      } else close();
      return;
    }
    if (busy) return;
    if (confirmation) {
      if (value === "y" || key.return) {
        setConfirmation(false);
        if (child) void control("stop", { executionId: child.executionId });
      } else if (value === "n") setConfirmation(false);
      return;
    }
    if (guidance) {
      if (
        value === "v" &&
        guidance.available &&
        guidance.revision !== null &&
        intent
      ) {
        const revision = guidance.revision;
        const available = guidance.available.id;
        void run(async (id, signal) => {
          await agent.adoptResearch(id, intent, revision, available);
          if (!signal.aborted) setGuidance(null);
        }).then((owned) => {
          if (owned) void refresh();
        });
      }
      return;
    }
    if (value === "x" && child) {
      setConfirmation(true);
      return;
    }
    if (history) {
      if (value === "r") void openHistory();
      return;
    }
    if (result) {
      if (value === "r") void openResult();
      return;
    }
    if (key.upArrow || key.downArrow) {
      const children = state?.children ?? [];
      const index = children.findIndex((c) => c.executionId === selected);
      const next =
        children[
          Math.max(
            0,
            Math.min(children.length - 1, index + (key.upArrow ? -1 : 1)),
          )
        ];
      if (next) setSelected(next.executionId);
    } else if (value === " " && child)
      setMembers((current) => {
        const next = new Set(current);
        if (next.has(child.executionId)) next.delete(child.executionId);
        else next.add(child.executionId);
        return next;
      });
    else if (value === "r") void refresh();
    else if (key.return) void openHistory();
    else if (value === "o") void openResult();
    else if ((value === "a" || value === "l") && members.size)
      void control("join", {
        id: crypto.randomUUID(),
        mode: value === "a" ? "any" : "all",
        executions: [...members],
      });
    else if (value === "j") setJoin((current) => current + 1);
    else if (value === "b" && captured)
      void control("background", { id: captured.id });
  }
  return {
    visible,
    state,
    selected,
    members,
    captured,
    child,
    history,
    result,
    guidance,
    confirmation,
    busy,
    error,
    input,
    inspect,
    inspectGuidance,
    dispatch,
    close,
  };
}
